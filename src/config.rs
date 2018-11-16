use crate::errors::*;
use crate::pull_list::repo::Repo;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use toml;

fn parse_config(path: &str) -> Config {
    let mut config_toml = String::new();
    let parsed_path = Path::new(path).canonicalize().unwrap();

    let mut file = match File::open(parsed_path) {
        Ok(file) => file,
        Err(e) => panic!("Could not find config file! [{:?}]", e),
    };

    file.read_to_string(&mut config_toml)
        .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

    let config: Config = match toml::from_str(&config_toml) {
        Ok(t) => t,
        Err(e) => panic!("Error while deserializing config [{:?}]", e),
    };

    config
}

#[derive(Deserialize, Clone, Debug)]
pub struct Project {
    pub name: String,
    pub id: String,
    pub milestones: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    // List of the repositories we need to evaluate
    pub repos: Vec<Repo>,
    // API token for GitHub
    pub github_token: String,
    // Name of the organisation in Zoho
    pub zoho_organisation: String,
    // API token for Zoho
    pub zoho_authtoken: String,
    // Projects in Zoho
    pub zoho_projects: Vec<Project>,
}

impl Config {
    fn construct(&mut self) -> Result<()> {
        let mut temp = ::std::mem::replace(&mut self.repos, Vec::new());
        for repo in &mut temp {
            repo.construct(self)?;
        }
        self.repos = temp;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Config {
        let mut config = parse_config("./config.toml");

        match config.construct() {
            Ok(()) => config,
            Err(e) => panic!("Couldn't construct configuration: {:?}", e),
        }
    }
}
