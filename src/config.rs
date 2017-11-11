use pull_list::repo::Repo;
use std::env;
use errors::*;

// TODO: This should be in a TOML config file (i.e. [[repo]]) parsed in Main.
// Hardcoding for now so I don't have to bother with a TOML parser upfront.

fn repo_list() -> Vec<Repo> {
    return vec![
        Repo {
            name: String::from("niciliketo/auction-frontend"),
            base: String::from("master"),
            last_release: None,
            pulls: None,
        },
        Repo {
            name: String::from("niciliketo/auction"),
            base: String::from("development"),
            last_release: None,
            pulls: None,
        },
    ];
}

fn project_list() -> Vec<Project> {
    return vec![
        Project {
            name: String::from("Market Dojo"),
            id: String::from("328792000000016009"),
            milestone: String::from("11.2.6"),
        },
        Project {
            name: String::from("Quick Quotes"),
            id: String::from("328792000012869177"),
            milestone: String::from("Phase 1 (Beta release)"),
        },
    ];
}

#[derive(Clone)]
pub struct Project {
    pub name: String,
    pub id: String,
    pub milestone: String,
}

#[derive(Clone)]
pub struct Config {
    // List of the repositories we need to evaluate
    pub repos: Option<Vec<Repo>>,
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
    fn construct(mut self) -> Result<Config> {
        let mut rl = vec![];
        for mut repo in repo_list() {
            repo.construct(&self)?;
            rl.push(repo);
        }
        self.repos = Some(rl);
        Ok(self)
    }
}

impl Default for Config {
    fn default() -> Config {
        let config = Config {
            repos: None,
            github_token: env::var("GITHUB_TOKEN").ok().unwrap(),
            zoho_organisation: String::from("marketdojo"),
            zoho_authtoken: env::var("ZOHO_AUTHTOKEN").ok().unwrap(),
            zoho_projects: project_list(),
        };

        config.construct().unwrap()
    }
}