#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate reqwest;
extern crate chrono;
extern crate hyper;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate inflector;

mod errors;
mod pull_list;
mod zoho_bugs;
mod config;

use errors::*;
use pull_list::{print_repo, issue_labels};
use pull_list::repo::Repo;
use zoho_bugs::{print_bugs, issue};
use config::{Config, Project};

fn labels(repos: &[Repo]) -> Vec<String> {
    let mut labels = vec![];
    for repo in repos {
        let mut lst = issue_labels(repo).unwrap();
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
        .map(|p| p.milestone.to_owned())
        .collect();
    let milestones = milestone_list.join(", ");
    println!(
        "We have released a new version of Market Dojo to live.\n\n\
        Please let your customers know if they are listed and you feel \
        the fixes will be relevant to them.\n\nThis includes development \
        of the {} milestones. A complete list of changes is attached.\n\n\
        Many thanks to the whole team who have worked incredibly hard \
        to make this release possible.\n",
        milestones
    );
    Ok(())
}

fn print_projects(labels: &[String], projects: Vec<Project>, config: &Config) -> Result<()> {
    for project in projects {
        println!("\n### Closed issues for {}, by customer:\n", project.name);
        let issues = issue::build_list(&project.id, project.milestone, labels.to_owned(), config)?;
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
    let repos = config.repos.clone().unwrap();
    let projects = config.zoho_projects.clone();
    let labels = labels(&repos);

    print_preamble(&config)?;
    print_projects(&labels, projects, &config)?;
    print_repos(repos)?;

    Ok(0)
}

quick_main!(run);
