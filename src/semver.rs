// versions and version ranges as mach's doc/language/manifest.md defines them

use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Vec<String>,
}

impl Version {
    // a release's version, with any `+build` metadata ignored
    pub fn parse(s: &str) -> Option<Version> {
        let s = s.split_once('+').map_or(s, |(v, _)| v);
        let (core, pre) = match s.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (s, None),
        };
        let mut parts = core.split('.');
        let v = Version {
            major: number(parts.next()?)?,
            minor: number(parts.next()?)?,
            patch: number(parts.next()?)?,
            pre: match pre {
                Some(pre) => identifiers(pre)?,
                None => Vec::new(),
            },
        };
        parts.next().is_none().then_some(v)
    }

    pub fn is_pre(&self) -> bool {
        !self.pre.is_empty()
    }

    fn release(&self) -> (u64, u64, u64) {
        (self.major, self.minor, self.patch)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.release()
            .cmp(&other.release())
            .then_with(|| match (self.is_pre(), other.is_pre()) {
                (false, false) => Ordering::Equal,
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (true, true) => cmp_pre(&self.pre, &other.pre),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn cmp_pre(a: &[String], b: &[String]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        let ord = match (x.parse::<u64>(), y.parse::<u64>()) {
            (Ok(x), Ok(y)) => x.cmp(&y),
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
            (Err(_), Err(_)) => x.cmp(y),
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    a.len().cmp(&b.len())
}

fn number(s: &str) -> Option<u64> {
    let digits = !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let canonical = s == "0" || !s.starts_with('0');
    (digits && canonical).then(|| s.parse().ok()).flatten()
}

fn identifiers(s: &str) -> Option<Vec<String>> {
    s.split('.')
        .map(|id| {
            let ok = !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
            ok.then(|| id.to_string())
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Caret,
    Tilde,
    Ge,
    Gt,
    Le,
    Lt,
    Eq,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Clause {
    op: Op,
    // the named version with missing components as 0
    version: Version,
    // how many of major, minor, patch the clause names
    given: usize,
}

impl Clause {
    fn parse(s: &str) -> Option<Clause> {
        let s = s.trim();
        let (op, rest) = [
            (">=", Op::Ge),
            ("<=", Op::Le),
            ("^", Op::Caret),
            ("~", Op::Tilde),
            (">", Op::Gt),
            ("<", Op::Lt),
            ("=", Op::Eq),
        ]
        .iter()
        .find_map(|(token, op)| s.strip_prefix(token).map(|rest| (*op, rest.trim_start())))?;
        if rest.contains('+') {
            return None;
        }
        let (core, pre) = match rest.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (rest, None),
        };
        let parts = core.split('.').map(number).collect::<Option<Vec<u64>>>()?;
        let given = parts.len();
        if !(1..=3).contains(&given)
            || (pre.is_some() && given != 3)
            || (op == Op::Eq && given != 3)
        {
            return None;
        }
        let at = |i: usize| parts.get(i).copied().unwrap_or(0);
        Some(Clause {
            op,
            version: Version {
                major: at(0),
                minor: at(1),
                patch: at(2),
                pre: match pre {
                    Some(pre) => identifiers(pre)?,
                    None => Vec::new(),
                },
            },
            given,
        })
    }

    // the exclusive upper bound of a caret or tilde clause
    fn upper(&self) -> Version {
        let v = &self.version;
        let (major, minor, patch) = match (self.op, self.given) {
            (Op::Tilde, 1) => (v.major + 1, 0, 0),
            (Op::Tilde, _) => (v.major, v.minor + 1, 0),
            (_, _) if v.major > 0 => (v.major + 1, 0, 0),
            (_, 1) => (1, 0, 0),
            (_, _) if v.minor > 0 => (0, v.minor + 1, 0),
            (_, 2) => (0, 1, 0),
            (_, _) => (0, 0, v.patch + 1),
        };
        Version {
            major,
            minor,
            patch,
            pre: Vec::new(),
        }
    }

    fn admits(&self, v: &Version) -> bool {
        let named = &self.version;
        match self.op {
            Op::Caret | Op::Tilde => v >= named && *v < self.upper(),
            Op::Ge => v >= named,
            Op::Gt => v > named,
            Op::Le => v <= named,
            Op::Lt => v < named,
            Op::Eq => v == named,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    clauses: Vec<Clause>,
}

impl Range {
    pub fn parse(s: &str) -> Option<Range> {
        let clauses = s
            .split(',')
            .map(Clause::parse)
            .collect::<Option<Vec<_>>>()?;
        Some(Range { clauses })
    }

    // every clause admits it, and a pre-release also needs a clause naming a pre-release of it
    pub fn admits(&self, v: &Version) -> bool {
        let pre_named = !v.is_pre()
            || self
                .clauses
                .iter()
                .any(|c| c.version.is_pre() && c.version.release() == v.release());
        pre_named && self.clauses.iter().all(|c| c.admits(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    fn admits(range: &str, version: &str) -> bool {
        Range::parse(range).unwrap().admits(&v(version))
    }

    #[test]
    fn versions_parse_and_order_by_semver() {
        assert_eq!(v("5.4.0+build.7"), v("5.4.0"));
        assert!(v("0.9.0") < v("0.19.0"));
        assert!(v("1.3.0-rc.1") < v("1.3.0"));
        assert!(v("1.3.0-alpha") < v("1.3.0-alpha.1"));
        assert!(v("1.3.0-alpha.1") < v("1.3.0-alpha.beta"));
        assert!(v("1.3.0-rc.2") < v("1.3.0-rc.10"));
        for bad in [
            "5.4",
            "5.4.0.1",
            "v5.4.0",
            "05.4.0",
            "5.4.x",
            "5.4.0-",
            "5.4.0-a..b",
            "",
        ] {
            assert_eq!(Version::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn caret_and_tilde_match_the_doc_tables() {
        let cases = [
            ("^1.2.3", "1.2.3", "2.0.0"),
            ("^1.2", "1.2.0", "2.0.0"),
            ("^1", "1.0.0", "2.0.0"),
            ("~1.2.3", "1.2.3", "1.3.0"),
            ("~1.2", "1.2.0", "1.3.0"),
            ("~1", "1.0.0", "2.0.0"),
            ("^0.4.2", "0.4.2", "0.5.0"),
            ("^0.4", "0.4.0", "0.5.0"),
            ("^0.0.3", "0.0.3", "0.0.4"),
            ("^0.0", "0.0.0", "0.1.0"),
            ("^0", "0.0.0", "1.0.0"),
        ];
        for (range, low, high) in cases {
            let r = Range::parse(range).unwrap();
            let (low, high) = (v(low), v(high));
            assert!(r.admits(&low), "{range} admits {low:?}");
            assert!(!r.admits(&high), "{range} excludes {high:?}");
            let below = Version {
                patch: low.patch.wrapping_sub(1),
                ..low.clone()
            };
            if low.patch > 0 {
                assert!(!r.admits(&below), "{range} excludes {below:?}");
            }
        }
        assert!(admits("~5.3.1", "5.3.9"));
        assert!(!admits("~5.3.1", "5.4.0"));
    }

    #[test]
    fn bounds_fill_missing_components_with_zero() {
        assert!(admits(">1.2", "1.2.1"));
        assert!(!admits(">1.2", "1.2.0"));
        assert!(admits("<=1.2", "1.2.0"));
        assert!(!admits("<=1.2", "1.2.1"));
        assert!(admits(">=5.4, <6", "5.9.9"));
        assert!(!admits(">=5.4, <6", "6.0.0"));
        assert!(admits("=5.4.0", "5.4.0"));
        assert!(!admits("=5.4.0", "5.4.1"));
    }

    #[test]
    fn a_range_is_the_intersection_of_its_clauses() {
        assert!(admits("^5.2, >=5.4", "5.4.0"));
        assert!(!admits("^5.2, >=5.4", "5.3.0"));
        assert!(admits(">= 5.4 ,< 6", "5.4.0"));
    }

    #[test]
    fn pre_releases_need_a_clause_naming_one_of_that_release() {
        assert!(!admits("^1.2", "1.3.0-rc.1"));
        assert!(admits(">=1.3.0-rc.1, <2", "1.3.0-rc.1"));
        assert!(admits(">=1.3.0-rc.1, <2", "1.3.0"));
        assert!(!admits(">=1.3.0-rc.1, <2", "1.4.0-rc.1"));
        assert!(!admits("<2", "2.0.0-rc.1"));
    }

    #[test]
    fn malformed_ranges_are_refused() {
        for bad in [
            "1.2",
            "banana",
            "^",
            "^1.2.3.4",
            "=1.2",
            "^1.2+meta",
            "^1.2-rc.1",
            "*",
            "^1 || ^2",
            "",
            "^1,",
        ] {
            assert_eq!(Range::parse(bad), None, "{bad}");
        }
    }
}
