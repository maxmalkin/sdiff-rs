//! Custom error types for SDIFF.

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Failed to read file {path}: {source}")]
    ReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid JSON in {path}: {source}")]
    JsonError {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Invalid YAML in {path}: {source}")]
    YamlError {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("Invalid TOML in {path}: {source}")]
    TomlError {
        path: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("Could not detect file format for {path}")]
    UnknownFormat { path: String },
}

#[derive(Debug, thiserror::Error)]
pub enum OutputError {
    #[error("Unknown output format: {format}")]
    UnknownFormat { format: String },

    #[error("Failed to serialize to JSON: {source}")]
    JsonSerializationError {
        #[source]
        source: serde_json::Error,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum SdiffError {
    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    Output(#[from] OutputError),

    #[error("Invalid configuration: {message}")]
    ConfigError { message: String },
}

impl ParseError {
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    pub fn read_error(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::ReadError {
            path: path.into(),
            source,
        }
    }

    pub fn json_error(path: impl Into<String>, source: serde_json::Error) -> Self {
        Self::JsonError {
            path: path.into(),
            source,
        }
    }

    pub fn yaml_error(path: impl Into<String>, source: serde_yaml::Error) -> Self {
        Self::YamlError {
            path: path.into(),
            source,
        }
    }

    pub fn toml_error(path: impl Into<String>, source: toml::de::Error) -> Self {
        Self::TomlError {
            path: path.into(),
            source,
        }
    }

    pub fn unknown_format(path: impl Into<String>) -> Self {
        Self::UnknownFormat { path: path.into() }
    }
}
