use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{program} is not installed")]
    NotInstalled { program: String },
    #[error("{program} could not start: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{program} timed out after {seconds}s")]
    Timeout { program: String, seconds: u64 },
    #[error("{program} failed ({status}): {stderr}")]
    CommandFailed {
        program: String,
        status: String,
        stderr: String,
    },
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("refusing to delete {0}: only paths inside ~/Library and ~/.cache may be deleted")]
    UnsafePath(PathBuf),
    #[error("unknown area: {0}")]
    UnknownArea(String),
    #[error("HOME must be an absolute path")]
    NoHome,
    #[error("could not parse {what}: {detail}")]
    Parse { what: &'static str, detail: String },
}

impl CoreError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

impl serde::Serialize for CoreError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CoreError>;
