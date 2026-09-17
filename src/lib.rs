mod install;

use std::fs;
use zed_extension_api::{
    self as zed, process::Command, settings::LspSettings, DownloadedFileType, GithubReleaseOptions,
    LanguageServerId, LanguageServerInstallationStatus, Result,
};

struct MachExtension {
    downloaded_binary: Option<String>,
}

impl MachExtension {
    // a release mls in the extension work dir, downloading the latest when needed
    fn downloaded_binary(&mut self, id: &LanguageServerId) -> Result<String> {
        if let Some(path) = &self.downloaded_binary {
            if is_file(path) {
                return Ok(path.clone());
            }
        }
        let path = resolve_download(id).inspect_err(|e| {
            zed::set_language_server_installation_status(
                id,
                &LanguageServerInstallationStatus::Failed(e.clone()),
            )
        })?;
        zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::None);
        self.downloaded_binary = Some(path.clone());
        Ok(path)
    }
}

fn resolve_download(id: &LanguageServerId) -> Result<String> {
    let (os, arch) = zed::current_platform();
    let target = install::target(os, arch).ok_or_else(|| {
        format!(
            "mach-lsp ships no mls binary for {os:?} {arch:?}. \
             Put mls on $PATH or set lsp.mls.binary.path in Zed settings"
        )
    })?;
    zed::set_language_server_installation_status(
        id,
        &LanguageServerInstallationStatus::CheckingForUpdate,
    );
    let release = zed::latest_github_release(
        install::REPO,
        GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    );
    match release {
        Ok(release) => install_release(id, &release, &target),
        // offline: keep using the newest release already installed
        Err(e) => installed_binary(&target)
            .ok_or_else(|| format!("could not fetch the latest {} release: {e}", install::REPO)),
    }
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
    let sums_url = url_of(install::SUMS)?;

    zed::set_language_server_installation_status(
        id,
        &LanguageServerInstallationStatus::Downloading,
    );
    let staging = install::staging_dir(version);
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|e| format!("could not create {staging}: {e}"))?;

    let sums_path = format!("{staging}/{}", install::SUMS);
    let archive_path = format!("{staging}/{asset_name}");
    zed::download_file(&sums_url, &sums_path, DownloadedFileType::Uncompressed)?;
    zed::download_file(
        &archive_url,
        &archive_path,
        DownloadedFileType::Uncompressed,
    )?;
    let sums = fs::read_to_string(&sums_path).map_err(|e| format!("reading {sums_path}: {e}"))?;
    let archive = fs::read(&archive_path).map_err(|e| format!("reading {archive_path}: {e}"))?;

    install::verify(&sums, &asset_name, &archive)?;
    let bytes = install::extract_binary(target.archive, &archive, target.binary)?;
    fs::write(format!("{staging}/{}", target.binary), bytes)
        .map_err(|e| format!("writing {}: {e}", target.binary))?;
    let _ = fs::remove_file(&sums_path);
    let _ = fs::remove_file(&archive_path);

    // the rename is the commit point, so a complete install dir always holds a verified binary
    let _ = fs::remove_dir_all(&dir);
    fs::rename(&staging, &dir).map_err(|e| format!("could not install {dir}: {e}"))?;
    zed::make_file_executable(&binary)?;
    prune_installs_except(&dir);
    Ok(binary)
}

fn installed_binary(target: &install::Target) -> Option<String> {
    let names = work_dir_names();
    let dir = install::newest_installed(names.iter().map(String::as_str))?;
    let binary = format!("{dir}/{}", target.binary);
    is_file(&binary).then_some(binary)
}

fn prune_installs_except(keep: &str) {
    for name in work_dir_names() {
        if name != keep && install::is_install_dir(&name) {
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
        MachExtension {
            downloaded_binary: None,
        }
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
                None => self.downloaded_binary(language_server_id)?,
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
