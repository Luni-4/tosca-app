use std::fmt;
use std::fs;
use std::net::Ipv4Addr;
use std::path::Path;

use serde::Deserialize;

const CONFIG_FILEPATH: &str = "tosca-app.toml";

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub(crate) enum Language {
    #[default]
    English,
    Italian,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Language {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Italian => "it",
        }
    }
}

const fn default_host() -> Ipv4Addr {
    Ipv4Addr::LOCALHOST
}

const fn default_port() -> u16 {
    8123
}

const fn default_language() -> Language {
    Language::English
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct Configuration {
    #[serde(default = "default_host")]
    pub(crate) host: Ipv4Addr,
    #[serde(default = "default_port")]
    pub(crate) port: u16,
    #[serde(default = "default_language")]
    pub(crate) language: Language,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            language: Language::default(),
        }
    }
}

impl Configuration {
    pub(crate) fn load() -> Self {
        let config_filepath = Path::new(CONFIG_FILEPATH);

        match fs::read_to_string(config_filepath) {
            Ok(contents) => match toml::from_str::<Configuration>(&contents) {
                Ok(config) => config,
                Err(parse_err) => Self::manage_error("Failed to parse", config_filepath, parse_err),
            },
            Err(io_err) => Self::manage_error("Failed to read", config_filepath, io_err),
        }
    }

    #[inline]
    fn manage_error(msg: &str, config_filepath: &Path, error: impl std::error::Error) -> Self {
        #[cfg(feature = "logging")]
        {
            tracing::error!("{msg} `{}`: {error}", config_filepath.display());
            tracing::info!("Using default configuration");
        }
        Configuration::default()
    }
}
