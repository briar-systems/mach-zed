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
        // 1. Check user-configured binary path via Zed LSP settings.
        //
        //    Users can set this in their settings.json:
        //    {
        //        "lsp": {
        //            "mls": {
        //                "binary": {
        //                    "path": "/path/to/mls",
        //                    "arguments": []
        //                }
        //            }
        //        }
        //    }
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

        // 2. Check if we previously found the binary and it still exists.
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

        // 3. Look for `mls` on the system PATH.
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
