// which mls release links a compiler the project's dependency closure accepts

use std::collections::{BTreeMap, HashSet, VecDeque};

use crate::semver::{Range, Version};

pub const RELEASES: &str = "RELEASES.json";

// every mls version mapped to the mach version it links, as mach-lsp publishes it
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    entries: Vec<(Version, String, Version)>,
}

impl Catalog {
    // `latest` is the version of the release the map was read from, which it must list
    pub fn parse(json: &str, latest: &str) -> Result<Catalog, String> {
        let catalog = Catalog::from_json(json)?;
        if !catalog.contains(latest) {
            return Err(format!(
                "mach-lsp {RELEASES} in release {latest} does not list {latest}"
            ));
        }
        Ok(catalog)
    }

    // a map already checked against its release, such as the saved copy
    pub fn from_json(json: &str) -> Result<Catalog, String> {
        let map: BTreeMap<String, String> = zed_extension_api::serde_json::from_str(json)
            .map_err(|e| format!("mach-lsp {RELEASES} is not a string map: {e}"))?;
        let mut entries = Vec::with_capacity(map.len());
        for (mls, mach) in map {
            let parsed = Version::parse(&mls).ok_or_else(|| {
                format!("mach-lsp {RELEASES} lists `{mls}`, which is not a version")
            })?;
            let linked = Version::parse(&mach).ok_or_else(|| {
                format!("mach-lsp {RELEASES} maps {mls} to `{mach}`, which is not a version")
            })?;
            entries.push((parsed, mls, linked));
        }
        entries.sort();
        Ok(Catalog { entries })
    }

    pub fn contains(&self, mls: &str) -> bool {
        self.entries.iter().any(|(_, m, _)| m == mls)
    }

    // the newest mls whose mach every range admits, else the newest mls, among those `usable` allows
    pub fn select(&self, ranges: &[Range], usable: impl Fn(&str) -> bool) -> Option<&str> {
        let candidates = || {
            self.entries
                .iter()
                .rev()
                .filter(|(v, mls, _)| !v.is_pre() && usable(mls))
        };
        candidates()
            .find(|(_, _, mach)| ranges.iter().all(|r| r.admits(mach)))
            .or_else(|| candidates().next())
            .map(|(_, mls, _)| mls.as_str())
    }
}

// one `[project].mach` in the closure, with the manifest chain that states it
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub chain: String,
    pub range: String,
}

// ranges stated by the root manifest and every manifest it reaches, read through `read`
pub fn requirements(mut read: impl FnMut(&str) -> Option<String>) -> Vec<Requirement> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([("mach.toml".to_string(), "root".to_string())]);
    while let Some((path, chain)) = queue.pop_front() {
        if !seen.insert(path.clone()) {
            continue;
        }
        let Some(table) = read(&path).and_then(|text| text.parse::<toml::Table>().ok()) else {
            continue;
        };
        let project = table.get("project").and_then(|p| p.as_table());
        if let Some(range) = project.and_then(|p| p.get("mach")).and_then(|m| m.as_str()) {
            found.push(Requirement {
                chain: chain.clone(),
                range: range.to_string(),
            });
        }
        let deps = table.get("dep").and_then(|d| d.as_table());
        for (id, dep) in deps.into_iter().flatten() {
            let manifest = match dep.get("path").and_then(|p| p.as_str()) {
                Some(dir) => match join(parent(&path), dir) {
                    Some(dir) => format!("{dir}mach.toml"),
                    None => continue,
                },
                None => format!("dep/{id}/mach.toml"),
            };
            queue.push_back((manifest, format!("{chain} -> {id}")));
        }
    }
    found
}

// the parsed ranges, or none at all when one does not parse: mls reports a malformed range itself
pub fn ranges(requirements: &[Requirement]) -> Vec<Range> {
    requirements
        .iter()
        .map(|r| Range::parse(&r.range))
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
}

fn parent(path: &str) -> &str {
    path.rfind('/').map_or("", |i| &path[..=i])
}

// a worktree-relative directory with a trailing `/`, or none when it leaves the worktree
fn join(base: &str, rel: &str) -> Option<String> {
    if rel.starts_with('/') || rel.contains(':') || rel.contains('\\') {
        return None;
    }
    let mut parts: Vec<&str> = base.split('/').filter(|p| !p.is_empty()).collect();
    for part in rel.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name),
        }
    }
    Some(parts.iter().map(|p| format!("{p}/")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const MAP: &str = r#"{"0.19.0": "5.2.1", "0.20.0": "5.4.0", "0.20.1": "5.4.0",
        "0.21.0-rc.1": "5.5.0", "0.21.0": "5.5.0", "1.0.0": "6.0.0"}"#;

    fn catalog() -> Catalog {
        Catalog::parse(MAP, "1.0.0").unwrap()
    }

    fn pick(ranges: &[&str]) -> Option<String> {
        let ranges: Vec<Range> = ranges.iter().map(|r| Range::parse(r).unwrap()).collect();
        catalog().select(&ranges, |_| true).map(str::to_string)
    }

    #[test]
    fn picks_the_newest_mls_whose_mach_the_ranges_admit() {
        assert_eq!(pick(&[]).as_deref(), Some("1.0.0"));
        assert_eq!(pick(&["^5.4"]).as_deref(), Some("0.21.0"));
        assert_eq!(pick(&["~5.4"]).as_deref(), Some("0.20.1"));
        assert_eq!(pick(&["^5.2", "<5.3"]).as_deref(), Some("0.19.0"));
        assert_eq!(
            pick(&["~5.3.1"]).as_deref(),
            Some("1.0.0"),
            "no release links 5.3, so the newest"
        );
        assert_eq!(pick(&["^6"]).as_deref(), Some("1.0.0"));
    }

    #[test]
    fn never_picks_a_pre_release_mls() {
        assert_eq!(pick(&[">=5.5.0, <5.6"]).as_deref(), Some("0.21.0"));
        let only_pre =
            Catalog::parse(r#"{"0.21.0-rc.1": "5.5.0", "0.20.0": "5.4.0"}"#, "0.20.0").unwrap();
        assert_eq!(
            only_pre.select(&[Range::parse("^5.5").unwrap()], |_| true),
            Some("0.20.0")
        );
    }

    #[test]
    fn select_can_be_limited_to_installed_versions() {
        let installed = ["0.19.0", "0.20.0"];
        let c = catalog();
        let pick = |r: &str| c.select(&[Range::parse(r).unwrap()], |m| installed.contains(&m));
        assert_eq!(pick("^5.4"), Some("0.20.0"));
        assert_eq!(pick("^6"), Some("0.20.0"));
        assert_eq!(c.select(&[], |_| false), None);
    }

    #[test]
    fn a_map_that_misses_its_own_release_or_is_malformed_is_a_defect() {
        assert!(Catalog::parse(MAP, "1.0.1")
            .unwrap_err()
            .contains("does not list 1.0.1"));
        assert!(Catalog::parse(r#"{"v0.20.1": "5.4.0"}"#, "0.20.1")
            .unwrap_err()
            .contains("not a version"));
        assert!(Catalog::parse(r#"{"0.20.1": "5.4"}"#, "0.20.1")
            .unwrap_err()
            .contains("not a version"));
        assert!(Catalog::parse(r#"{"0.20.1": 5}"#, "0.20.1")
            .unwrap_err()
            .contains("not a string map"));
        assert!(Catalog::parse("[]", "0.20.1").is_err());
        assert!(catalog().contains("0.20.1") && !catalog().contains("0.20.2"));
    }

    fn tree(files: &[(&str, &str)]) -> HashMap<String, String> {
        files
            .iter()
            .map(|(p, t)| (p.to_string(), t.to_string()))
            .collect()
    }

    fn reqs(files: &HashMap<String, String>) -> Vec<(String, String)> {
        requirements(|p| files.get(p).cloned())
            .into_iter()
            .map(|r| (r.chain, r.range))
            .collect()
    }

    #[test]
    fn walks_git_and_path_dependencies_through_the_flat_dep_dir() {
        let files = tree(&[
            (
                "mach.toml",
                "[project]\nid = \"app\"\nmach = \"^5.4\"\n\n[dep.std]\ngit = \"https://x/std\"\nversion = \"^4\"\n\n[dep.gfx]\npath = \"libs/gfx\"\n",
            ),
            ("dep/std/mach.toml", "[project]\nid = \"std\"\nmach = \">=5.3\"\n"),
            (
                "libs/gfx/mach.toml",
                "[project]\nid = \"gfx\"\nmach = \"<6\"\n\n[dep.glfw]\ngit = \"https://x/glfw\"\nref = \"tag/v1\"\n\n[dep.util]\npath = \"../util\"\n",
            ),
            ("dep/glfw/mach.toml", "[project]\nid = \"glfw\"\nmach = \"~5.4\"\n\n[dep.std]\ngit = \"https://x/std\"\n"),
            ("libs/util/mach.toml", "[project]\nid = \"util\"\n"),
        ]);
        assert_eq!(
            reqs(&files),
            [
                ("root", "^5.4"),
                ("root -> gfx", "<6"),
                ("root -> std", ">=5.3"),
                ("root -> gfx -> glfw", "~5.4"),
            ]
            .map(|(c, r)| (c.to_string(), r.to_string()))
        );
    }

    #[test]
    fn missing_unreadable_and_escaping_manifests_are_skipped() {
        assert!(reqs(&tree(&[])).is_empty());
        assert!(reqs(&tree(&[("mach.toml", "[project\nmach = \"^5\"")])).is_empty());
        let files = tree(&[
            (
                "mach.toml",
                "[project]\nmach = \"^5.4\"\n[dep.a]\ngit = \"x\"\n[dep.b]\npath = \"../outside\"\n[dep.c]\npath = \"/abs\"\n[dep.d]\npath = \"./d\"\n",
            ),
            ("d/mach.toml", "[project]\nmach = \"^5\"\n[dep.self]\npath = \".\"\n"),
        ]);
        assert_eq!(
            reqs(&files),
            [("root", "^5.4"), ("root -> d", "^5")].map(|(c, r)| (c.to_string(), r.to_string()))
        );
    }

    #[test]
    fn one_malformed_range_drops_every_constraint() {
        let good = Requirement {
            chain: "root".into(),
            range: "^5.4".into(),
        };
        let bad = Requirement {
            chain: "root -> a".into(),
            range: "banana".into(),
        };
        assert_eq!(ranges(std::slice::from_ref(&good)).len(), 1);
        assert!(ranges(&[good, bad]).is_empty());
    }
}
