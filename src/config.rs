use crate::pull_list::repo::Repo;
use color_eyre::{eyre::WrapErr, Result};
use serde_derive::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn parse_config(path: &str) -> Result<Config> {
    let mut config_toml = String::new();
    let parsed_path = Path::new(path)
        .canonicalize()
        .wrap_err("Failed to parse path to config file")?;

    let mut file = File::open(parsed_path).wrap_err("Could not find config file: ")?;

    file.read_to_string(&mut config_toml)
        .wrap_err("Error while reading config: ")?;

    toml::from_str(&config_toml).wrap_err("Error while deserializing config: ")
}

#[derive(Deserialize, Clone, Debug)]
pub struct Project {
    pub name: String,
    pub id: String,
    pub milestones: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    /// List of the repositories we need to evaluate
    #[serde(default)]
    pub repos: Vec<Repo>,
    /// API token for GitHub
    pub github_token: String,
    /// Name of the organisation in Zoho
    pub zoho_portal_name: String,
    /// Client ID for Zoho OAuth
    pub zoho_client_id: String,
    /// Client Secret for Zoho OAuth
    pub zoho_client_secret: String,
    /// Projects in Zoho
    #[serde(default)]
    pub zoho_projects: Vec<Project>,
}
