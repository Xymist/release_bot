#![feature(type_ascription)]

extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate hyper;
extern crate inflector;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate zohohorrorshow;

mod config;
mod errors;
mod pull_list;
mod zoho_bugs;

use config::{Config, Project};
use errors::*;
use pull_list::repo::Repo;
use pull_list::{issue_labels, print_repo};
use zoho_bugs::{issue, print_bugs};

const PREAMBLE: &'static str = "Hi everyone,\n
We have released a new version of Market Dojo to live.\n
Please let your customers know if they are listed and you feel the fixes will be relevant to them.
A complete list of changes is attached.\n
Many thanks to the whole team who have worked incredibly hard to make this release possible.\n";

fn labels(repos: &[Repo]) -> Vec<String> {
    let mut labels = vec![];
    for repo in repos {
        let mut lst = issue_labels(repo).unwrap_or_default();
        labels.append(&mut lst);
    }
    labels.sort();
    labels.dedup();
    labels
}

fn print_preamble(config: &Config) -> Result<()> {
    let milestone_list: Vec<String> = config
        .zoho_projects
        .iter()
        .map(|p| p.milestones.join(", "))
        .collect();
    let milestones = milestone_list.join(", ");
    println!("{}", PREAMBLE);
    println!(
        "\nThis includes development of the {} milestone(s).\n\n",
        milestones
    );
    Ok(())
}

fn print_projects(labels: &[String], projects: Vec<Project>, config: &Config) -> Result<()> {
    for project in projects {
        println!("\n## Closed issues for {}\n", project.name);
        println!("\n### Customer Issues:\n");
        let issues = issue::build_list(
            project.id.parse::<i64>()?,
            project.milestones,
            labels.to_owned(),
            config,
        )?;
        print_bugs(issues)?;
    }
    Ok(())
}

fn print_repos(repos: Vec<Repo>) -> Result<()> {
    for repo in repos {
        print_repo(repo)?;
    }
    Ok(())
}

fn run() -> Result<i32> {
    let config = Config::default();
    let repos = config.repos.clone();
    let projects = config.zoho_projects.clone();
    let labels = labels(&repos);

    print_preamble(&config)?;
    print_projects(&labels, projects, &config)?;
    print_repos(repos)?;

    Ok(0)
}

quick_main!(run);
