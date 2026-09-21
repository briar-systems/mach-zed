mod compat;
mod install;
mod semver;

use std::collections::HashSet;
use std::fs;
use std::time::SystemTime;

use compat::Catalog;
use zed_extension_api::{
    self as zed, process::Command, settings::LspSettings, DownloadedFileType, GithubReleaseOptions,
    LanguageServerId, LanguageServerInstallationStatus, Result,
};

// the last catalog fetched, kept for choosing among installed versions offline
const SAVED_CATALOG: &str = "releases.json";

#[derive(Default)]
struct MachExtension {
    // the release catalog, fetched once per session
    catalog: Option<Catalog>,
    // install dirs this session has handed out, which pruning never removes
    used: HashSet<String>,
}

impl MachExtension {
    // a release mls in the extension work dir that links a compiler the worktree's project accepts
    fn downloaded_binary(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        let path = self.resolve_download(id, worktree).inspect_err(|e| {
            zed::set_language_server_installation_status(
                id,
                &LanguageServerInstallationStatus::Failed(e.clone()),
            )
        })?;
        zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::None);
        Ok(path)
    }

    fn resolve_download(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        let (os, arch) = zed::current_platform();
        let target = install::target(os, arch).ok_or_else(|| {
            format!(
                "mach-lsp ships no mls binary for {os:?} {arch:?}. \
                 Put mls on $PATH or set lsp.mls.binary.path in Zed settings"
            )
        })?;
        let requirements = compat::requirements(|path| worktree.read_text_file(path).ok());
        let ranges = compat::ranges(&requirements);

        let binary = match self.catalog(id)? {
            Ok(catalog) => {
                let version = catalog
                    .select(&ranges, |_| true)
                    .ok_or("mach-lsp publishes no release to install")?
                    .to_string();
                install_version(id, &version, &target)?
            }
            // offline: choose among what is already installed
            Err(e) => installed_binary(&target, &ranges).ok_or_else(|| {
                format!("could not fetch the {} release catalog: {e}", install::REPO)
            })?,
        };
        let dir = binary.split('/').next().unwrap_or_default().to_string();
        let _ = fs::write(format!("{dir}/{}", install::LAST_USED), "");
        self.used.insert(dir);
        prune_installs(&self.used);
        Ok(binary)
    }

    // the outer error is a defect in what mach-lsp published, the inner one an unreachable GitHub
    fn catalog(&mut self, id: &LanguageServerId) -> Result<std::result::Result<&Catalog, String>> {
        if self.catalog.is_none() {
            zed::set_language_server_installation_status(
                id,
                &LanguageServerInstallationStatus::CheckingForUpdate,
            );
            match fetch_catalog()? {
                Ok(catalog) => self.catalog = Some(catalog),
                Err(unreachable) => return Ok(Err(unreachable)),
            }
        }
        Ok(Ok(self.catalog.as_ref().expect("catalog was just set")))
    }
}

// RELEASES.json from the latest release, saved for offline starts
fn fetch_catalog() -> Result<std::result::Result<Catalog, String>> {
    let latest = zed::latest_github_release(
        install::REPO,
        GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    );
    let latest = match latest {
        Ok(latest) => latest,
        Err(unreachable) => return Ok(Err(unreachable)),
    };
    let version = install::version_of(&latest.version);
    let asset = latest
        .assets
        .iter()
        .find(|a| a.name == compat::RELEASES)
        .ok_or_else(|| {
            format!(
                "mach-lsp {} publishes no {} asset",
                latest.version,
                compat::RELEASES
            )
        })?;
    let partial = format!("{SAVED_CATALOG}.partial");
    if let Err(unreachable) = zed::download_file(
        &asset.download_url,
        &partial,
        DownloadedFileType::Uncompressed,
    ) {
        return Ok(Err(unreachable));
    }
    let json = fs::read_to_string(&partial).map_err(|e| format!("reading {partial}: {e}"));
    let _ = fs::remove_file(&partial);
    let json = json?;
    let catalog = Catalog::parse(&json, version)?;
    let _ = fs::write(SAVED_CATALOG, &json);
    Ok(Ok(catalog))
}

fn install_version(
    id: &LanguageServerId,
    version: &str,
    target: &install::Target,
) -> Result<String> {
    let binary = format!("{}/{}", install::install_dir(version), target.binary);
    if is_file(&binary) {
        return Ok(binary);
    }
    let release = zed::github_release_by_tag_name(install::REPO, &format!("v{version}"))?;
    install_release(id, &release, target)
}

fn install_release(
    id: &LanguageServerId,
    release: &zed::GithubRelease,
    target: &install::Target,
) -> Result<String> {
    let version = install::version_of(&release.version);
    let dir = install::install_dir(version);
    let binary = format!("{dir}/{}", target.binary);
    if is_file(&binary) {
        return Ok(binary);
    }

    let asset_name = install::asset_name(version, target);
    let url_of = |name: &str| {
        release
            .assets
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.download_url.clone())
            .ok_or_else(|| format!("mach-lsp {} has no {name} asset", release.version))
    };
    let archive_url = url_of(&asset_name)?;

    zed::set_language_server_installation_status(
        id,
        &LanguageServerInstallationStatus::Downloading,
    );
    let staging = install::staging_dir(version);
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|e| format!("could not create {staging}: {e}"))?;

    zed::download_file(&archive_url, &staging, target.archive.file_type())?;
    if !is_file(&format!("{staging}/{}", target.binary)) {
        return Err(format!("{asset_name} has no {}", target.binary));
    }

    // the rename is the commit point, so a complete install dir always holds a whole binary
    let _ = fs::remove_dir_all(&dir);
    fs::rename(&staging, &dir).map_err(|e| format!("could not install {dir}: {e}"))?;
    zed::make_file_executable(&binary)?;
    Ok(binary)
}

// the best installed binary, chosen with the saved catalog when there is one
fn installed_binary(target: &install::Target, ranges: &[semver::Range]) -> Option<String> {
    let dirs: Vec<String> = work_dir_names()
        .into_iter()
        .filter(|name| is_file(&format!("{name}/{}", target.binary)))
        .collect();
    let saved = fs::read_to_string(SAVED_CATALOG)
        .ok()
        .and_then(|json| Catalog::from_json(&json).ok());
    let dir = match saved {
        Some(catalog) => install::install_dir(
            catalog.select(ranges, |mls| dirs.contains(&install::install_dir(mls)))?,
        ),
        None => install::newest_installed(dirs.iter().map(String::as_str))?.to_string(),
    };
    Some(format!("{dir}/{}", target.binary))
}

fn prune_installs(in_use: &HashSet<String>) {
    let now = SystemTime::now();
    for name in work_dir_names() {
        let last_used = fs::metadata(format!("{name}/{}", install::LAST_USED))
            .or_else(|_| fs::metadata(&name))
            .and_then(|m| m.modified())
            .ok();
        let idle = last_used.and_then(|t| now.duration_since(t).ok());
        if install::prunable(&name, in_use.contains(&name), idle) {
            let _ = fs::remove_dir_all(&name);
        }
    }
}

fn work_dir_names() -> Vec<String> {
    fs::read_dir(".")
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).is_ok_and(|m| m.is_file())
}

impl zed::Extension for MachExtension {
    fn new() -> Self {
        MachExtension::default()
    }

    // resolution order: lsp.mls.binary settings, then $PATH, then a downloaded release
    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Command> {
        let binary = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .and_then(|s| s.binary);
        let args = binary
            .as_ref()
            .and_then(|b| b.arguments.clone())
            .unwrap_or_default();

        let command = match binary.and_then(|b| b.path) {
            Some(path) => path,
            None => match worktree.which("mls") {
                Some(path) => path,
                None => self.downloaded_binary(language_server_id, worktree)?,
            },
        };
        Ok(Command {
            command,
            args,
            env: Default::default(),
        })
    }
}

zed::register_extension!(MachExtension);
