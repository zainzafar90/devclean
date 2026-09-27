use std::path::PathBuf;

use crate::error::{CoreError, Result};

#[derive(Debug, Clone)]
pub struct Context {
    pub home: PathBuf,
    pub config_file: PathBuf,
}

impl Context {
    pub fn from_env() -> Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|home| home.is_absolute() && home.parent().is_some())
            .ok_or(CoreError::NoHome)?;
        let config_file = std::env::var_os("DEVCLEAN_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config/devclean/config"));
        Ok(Self { home, config_file })
    }

    pub fn log_file(&self) -> PathBuf {
        self.home.join("Library/Logs/devclean.log")
    }

    pub fn tilde(&self, path: &std::path::Path) -> String {
        match path.strip_prefix(&self.home) {
            Ok(rest) => format!("~/{}", rest.display()),
            Err(_) => path.display().to_string(),
        }
    }
}
