use std::fs;
use zed_extension_api::{self as zed, process::Command, settings::LspSettings, Result};

struct MachExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for MachExtension {
    fn new() -> Self {
        MachExtension {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Command> {
        // user-configured path from lsp.mls.binary in settings.json
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .ok()
            .unwrap_or_default();

        if let Some(binary_settings) = &lsp_settings.binary {
            if let Some(path) = &binary_settings.path {
                let args = binary_settings.arguments.clone().unwrap_or_default();
                return Ok(Command {
                    command: path.clone(),
                    args,
                    env: Default::default(),
                });
            }
        }

        // previously resolved path, if it still exists
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok() {
                return Ok(Command {
                    command: path.clone(),
                    args: vec![],
                    env: Default::default(),
                });
            }
            self.cached_binary_path = None;
        }

        if let Some(path) = worktree.which("mls") {
            self.cached_binary_path = Some(path.clone());
            return Ok(Command {
                command: path,
                args: vec![],
                env: Default::default(),
            });
        }

        Err("mls not found. Install it and add it to your $PATH, \
             or set the binary path in Zed settings under lsp.mls.binary.path"
            .into())
    }
}

zed::register_extension!(MachExtension);
