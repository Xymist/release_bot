#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
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
use zoho_bugs::{print_bugs, issue};
use config::Config;
use dotenv::dotenv;

fn run() -> Result<i32> {
    dotenv().ok();
    let config = Config::default();
    let repos = config.repos.clone().unwrap();
    let projects = config.zoho_projects.clone();
    let mut labels = vec![];
    for repo in &repos {
        let mut lst = issue_labels(repo).unwrap();
        labels.append(&mut lst);
    }
    labels.sort();
    labels.dedup();
    println!("## Issues solved in this release:");
    for project in projects {
        let issues = issue::build_list(project.milestone, labels.clone(), &config)?;
        print_bugs(issues)?;
    }
    for repo in repos {
        print_repo(repo)?;
    }
    Ok(0)
}

quick_main!(run);
