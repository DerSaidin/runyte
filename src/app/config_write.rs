// SPDX-License-Identifier: MPL-2.0

//! Writing the effective configuration out as a complete YAML file.
//!
//! What is written is the saved configuration, [`App::persisted_config`],
//! rather than the running one. The two differ only for a setting read before
//! the editor existed and changed since: the running editor keeps its launch
//! value, but the file should carry the choice that was made, or writing it
//! back over the loaded file would quietly undo that choice.

use std::path::PathBuf;

use super::App;
use crate::settings::{SettingError, write_config};

impl App {
    /// Writes the configuration to `path`, or to the per-user configuration
    /// file when no path is given. An existing file is kept unless `replace`.
    pub(super) fn write_configuration(&mut self, path: Option<PathBuf>, replace: bool) {
        let Some(target) = path
            .map(|path| self.resolve_working_path(path))
            .or_else(|| self.default_config_target.clone())
        else {
            self.action_failed(
                "no configuration directory is known; set XDG_CONFIG_HOME or HOME, or give :write-config a path",
            );
            return;
        };
        match write_config(&target, &self.persisted_config, replace) {
            Ok(()) => self.status(format!("wrote configuration to {}", target.display())),
            Err(SettingError::AlreadyExists(path)) => self.action_warning(
                "Configuration not written",
                format!(
                    "{} already exists · :write-config! replaces it",
                    path.display()
                ),
            ),
            Err(error) => {
                self.error_from("Runyte", "Configuration write failed", error.to_string())
            }
        }
    }
}
