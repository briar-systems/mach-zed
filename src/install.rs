use zed_extension_api::{Architecture, DownloadedFileType, Os};

pub const REPO: &str = "briar-systems/mach-lsp";
pub const DIR_PREFIX: &str = "mls-";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Archive {
    TarGz,
    Zip,
}

impl Archive {
    fn ext(self) -> &'static str {
        match self {
            Archive::TarGz => "tar.gz",
            Archive::Zip => "zip",
        }
    }

    // the host extracts the archive into the download path
    pub fn file_type(self) -> DownloadedFileType {
        match self {
            Archive::TarGz => DownloadedFileType::GzipTar,
            Archive::Zip => DownloadedFileType::Zip,
        }
    }
}

// one shipped platform, named as mach-lsp's release asset contract names it
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub platform: &'static str,
    pub archive: Archive,
    pub binary: &'static str,
}

const TARGETS: &[(Os, Architecture, Target)] = &[
    (Os::Linux, Architecture::X8664, unix("x86_64-linux")),
    (Os::Linux, Architecture::Aarch64, unix("aarch64-linux")),
    (Os::Mac, Architecture::X8664, unix("x86_64-darwin")),
    (Os::Mac, Architecture::Aarch64, unix("aarch64-darwin")),
    (
        Os::Windows,
        Architecture::X8664,
        Target {
            platform: "x86_64-windows",
            archive: Archive::Zip,
            binary: "mls.exe",
        },
    ),
];

const fn unix(platform: &'static str) -> Target {
    Target {
        platform,
        archive: Archive::TarGz,
        binary: "mls",
    }
}

pub fn target(os: Os, arch: Architecture) -> Option<Target> {
    TARGETS
        .iter()
        .find(|(o, a, _)| *o == os && *a == arch)
        .map(|(_, _, t)| *t)
}

// release tags carry a leading `v`, asset names do not
pub fn version_of(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

pub fn asset_name(version: &str, target: &Target) -> String {
    format!("mls-{version}-{}.{}", target.platform, target.archive.ext())
}

pub fn install_dir(version: &str) -> String {
    format!("{DIR_PREFIX}{version}")
}

pub fn staging_dir(version: &str) -> String {
    format!("{}.partial", install_dir(version))
}

pub fn is_install_dir(name: &str) -> bool {
    name.starts_with(DIR_PREFIX)
}

// written into an install dir each time it is handed out, so its mtime says when it was last used
pub const LAST_USED: &str = ".last-used";

// installs unused for this long are removed
pub const RETENTION: std::time::Duration = std::time::Duration::from_secs(30 * 24 * 60 * 60);

// whether a work dir entry should be removed, given its age since last use when known
pub fn prunable(name: &str, in_use: bool, idle: Option<std::time::Duration>) -> bool {
    if !is_install_dir(name) || in_use {
        return false;
    }
    name.ends_with(".partial") || idle.is_some_and(|idle| idle > RETENTION)
}

// the newest complete install among work dir entry names
pub fn newest_installed<'a>(names: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    names
        .into_iter()
        .filter_map(|name| Some((parse_version(name.strip_prefix(DIR_PREFIX)?)?, name)))
        .max()
        .map(|(_, name)| name)
}

fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let mut parts = s.split('.').map(|p| p.parse::<u64>().ok());
    let v = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_platform_maps_to_its_asset() {
        let cases = [
            (
                Os::Linux,
                Architecture::X8664,
                "mls-0.19.0-x86_64-linux.tar.gz",
            ),
            (
                Os::Linux,
                Architecture::Aarch64,
                "mls-0.19.0-aarch64-linux.tar.gz",
            ),
            (
                Os::Mac,
                Architecture::X8664,
                "mls-0.19.0-x86_64-darwin.tar.gz",
            ),
            (
                Os::Mac,
                Architecture::Aarch64,
                "mls-0.19.0-aarch64-darwin.tar.gz",
            ),
            (
                Os::Windows,
                Architecture::X8664,
                "mls-0.19.0-x86_64-windows.zip",
            ),
        ];
        for (os, arch, asset) in cases {
            let t = target(os, arch).unwrap();
            assert_eq!(asset_name(version_of("v0.19.0"), &t), asset);
        }
        assert_eq!(
            target(Os::Windows, Architecture::X8664).unwrap().binary,
            "mls.exe"
        );
        assert_eq!(
            target(Os::Linux, Architecture::X8664).unwrap().binary,
            "mls"
        );
    }

    #[test]
    fn unshipped_platforms_have_no_target() {
        assert_eq!(target(Os::Linux, Architecture::X86), None);
        assert_eq!(target(Os::Windows, Architecture::Aarch64), None);
        assert_eq!(target(Os::Windows, Architecture::X86), None);
    }

    #[test]
    fn prunes_abandoned_staging_and_long_idle_installs_only() {
        let day = std::time::Duration::from_secs(24 * 60 * 60);
        assert!(prunable("mls-0.20.0.partial", false, Some(day)));
        assert!(prunable("mls-0.19.0", false, Some(RETENTION + day)));
        assert!(!prunable("mls-0.19.0", false, Some(day)));
        assert!(!prunable("mls-0.19.0", false, None));
        assert!(!prunable("mls-0.19.0", true, Some(RETENTION + day)));
        assert!(!prunable("releases.json", false, Some(RETENTION + day)));
        assert!(!prunable("grammars", false, Some(RETENTION + day)));
    }

    #[test]
    fn version_strips_only_a_leading_v() {
        assert_eq!(version_of("v0.19.0"), "0.19.0");
        assert_eq!(version_of("0.19.0"), "0.19.0");
    }

    #[test]
    fn newest_installed_orders_numerically_and_skips_partials() {
        let names = [
            "mls-0.9.0",
            "mls-0.19.0",
            "mls-0.20.0.partial",
            "mls-latest",
            "grammars",
        ];
        assert_eq!(newest_installed(names), Some("mls-0.19.0"));
        assert_eq!(newest_installed(["mls-0.20.0.partial"]), None);
    }
}
