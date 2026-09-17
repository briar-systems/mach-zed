use std::io::{Cursor, Read};

use sha2::{Digest, Sha256};
use zed_extension_api::{Architecture, Os};

pub const REPO: &str = "briar-systems/mach-lsp";
pub const SUMS: &str = "SHA256SUMS";
const DIR_PREFIX: &str = "mls-";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Archive {
    TarGz,
    Zip,
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
    let ext = match target.archive {
        Archive::TarGz => "tar.gz",
        Archive::Zip => "zip",
    };
    format!("mls-{version}-{}.{ext}", target.platform)
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

// checks `archive` against its line in a `sha256sum` formatted listing
pub fn verify(sums: &str, asset: &str, archive: &[u8]) -> Result<(), String> {
    let expected = sums
        .lines()
        .filter_map(|line| line.split_once(char::is_whitespace))
        .find(|(_, name)| {
            let name = name.trim_start();
            name.strip_prefix('*').unwrap_or(name) == asset
        })
        .map(|(hash, _)| hash.to_ascii_lowercase())
        .ok_or_else(|| format!("{SUMS} has no entry for {asset}"))?;
    let actual: String = Sha256::digest(archive)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "checksum mismatch for {asset}: expected {expected}, got {actual}"
        ))
    }
}

// reads the top-level `binary` entry out of a release archive
pub fn extract_binary(archive: Archive, bytes: &[u8], binary: &str) -> Result<Vec<u8>, String> {
    let found = match archive {
        Archive::TarGz => extract_tar_gz(bytes, binary).map_err(|e| e.to_string()),
        Archive::Zip => extract_zip(bytes, binary).map_err(|e| e.to_string()),
    }
    .map_err(|e| format!("could not read the mls archive: {e}"))?;
    found.ok_or_else(|| format!("the mls archive has no {binary}"))
}

fn extract_tar_gz(bytes: &[u8], binary: &str) -> std::io::Result<Option<Vec<u8>>> {
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if entry.header().entry_type().is_file() && is_top_level(&path.to_string_lossy(), binary) {
            let mut out = Vec::new();
            entry.read_to_end(&mut out)?;
            return Ok(Some(out));
        }
    }
    Ok(None)
}

fn extract_zip(bytes: &[u8], binary: &str) -> Result<Option<Vec<u8>>, zip::result::ZipError> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        if file.is_file() && is_top_level(file.name(), binary) {
            let mut out = Vec::new();
            file.read_to_end(&mut out)?;
            return Ok(Some(out));
        }
    }
    Ok(None)
}

fn is_top_level(path: &str, binary: &str) -> bool {
    path.strip_prefix("./").unwrap_or(path) == binary
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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

    fn sums_for(asset: &str, data: &[u8]) -> String {
        let hash: String = Sha256::digest(data)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        format!("0000  other.tar.gz\n{hash}  {asset}\n")
    }

    #[test]
    fn verify_accepts_a_matching_archive() {
        let sums = sums_for("a.tar.gz", b"payload");
        assert_eq!(verify(&sums, "a.tar.gz", b"payload"), Ok(()));
        let binary_mode = sums.replace("  a.tar.gz", " *a.tar.gz");
        assert_eq!(verify(&binary_mode, "a.tar.gz", b"payload"), Ok(()));
    }

    #[test]
    fn verify_rejects_a_tampered_or_unlisted_archive() {
        let sums = sums_for("a.tar.gz", b"payload");
        assert!(verify(&sums, "a.tar.gz", b"tampered")
            .unwrap_err()
            .contains("checksum mismatch"));
        assert!(verify(&sums, "b.tar.gz", b"payload")
            .unwrap_err()
            .contains("no entry"));
    }

    fn tar_gz(files: &[(&str, &[u8])]) -> Vec<u8> {
        let gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        let mut tar = tar::Builder::new(gz);
        for (name, data) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            tar.append_data(&mut header, name, *data).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap()
    }

    fn zip_of(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default();
        for (name, data) in files {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn extracts_the_binary_from_a_tarball() {
        let archive = tar_gz(&[("LICENSE", b"mit"), ("mls", b"elf")]);
        assert_eq!(
            extract_binary(Archive::TarGz, &archive, "mls").unwrap(),
            b"elf"
        );
        let dotted = tar_gz(&[("./mls", b"elf")]);
        assert_eq!(
            extract_binary(Archive::TarGz, &dotted, "mls").unwrap(),
            b"elf"
        );
    }

    #[test]
    fn extracts_the_binary_from_a_zip() {
        let archive = zip_of(&[("LICENSE", b"mit"), ("mls.exe", b"pe")]);
        assert_eq!(
            extract_binary(Archive::Zip, &archive, "mls.exe").unwrap(),
            b"pe"
        );
    }

    #[test]
    fn a_nested_or_missing_binary_is_an_error() {
        let nested = tar_gz(&[("bin/mls", b"elf")]);
        assert!(extract_binary(Archive::TarGz, &nested, "mls")
            .unwrap_err()
            .contains("has no mls"));
        assert!(extract_binary(Archive::Zip, b"not a zip", "mls.exe")
            .unwrap_err()
            .contains("could not read"));
    }
}
