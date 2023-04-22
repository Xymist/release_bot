#![warn(missing_docs)]
#![deny(
    unused_imports,
    rust_2018_idioms,
    rust_2018_compatibility,
    unsafe_code,
    clippy::all,
    dead_code
)]

//! This crate is a documentation generation crate for single releases of Market Dojo; it accesses both
//! GitHub and the Zoho Projects API to retrieve data.

mod config;
mod errors;
mod pull_list;
mod zoho_bugs;

use crate::config::{Config, Project};
use crate::errors::*;
use crate::pull_list::{format_repo, repo::Repo};
use crate::zoho_bugs::{classify_actions, issue, merge_actions, task, write_actions, zh_client};

use std::fmt::Write as _;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::sync::Arc;
use std::thread;

fn format_projects(projects: Vec<Project>, config: &Config) -> Result<String> {
    let mut output: String = "".to_owned();
    for project in projects {
        write!(
            output,
            "\n## Closed Tickets and Tasks for {}\n\n",
            project.name
        )
        .unwrap();

        let client = Arc::new(zh_client(&project, config)?);
        let sync_project = Arc::new(project.clone());

        let task_client = Arc::clone(&client);
        let task_project = Arc::clone(&sync_project);

        let issue_client = Arc::clone(&client);
        let issue_project = Arc::clone(&sync_project);

        let task_thread = thread::spawn(move || -> Result<Vec<zoho_bugs::Action>> {
            task::build_list(&task_client, &task_project.milestones)
        });
        let issue_thread = thread::spawn(move || -> Result<Vec<zoho_bugs::Action>> {
            issue::build_list(&issue_client, &issue_project.milestones)
        });

        let tasks = task_thread
            .join()
            .expect("Task list builder thread panicked: ")
            .expect("Task list builder failed to find any tasks: ");
        let issues = issue_thread
            .join()
            .expect("Issue list builder thread panicked: ")
            .expect("Issue list builder failed to find any issues: ");

        output.push_str(&write_actions(merge_actions(
            classify_actions(tasks),
            classify_actions(issues),
        )));
    }

    Ok(output)
}

fn format_repos(repos: Vec<Repo>, config: &Config) -> String {
    let mut output: String = "".to_owned();
    let mut children = Vec::new();
    let sync_config = Arc::new(config.clone());

    for repo in repos.into_iter() {
        let conf = Arc::clone(&sync_config);
        children.push(thread::spawn(move || -> String {
            format_repo(repo, &conf)
        }));
    }

    for child in children {
        output.push_str(&child.join().expect("Repo formatting thread failed: "));
    }

    output
}

fn write_output(config: &Config, projects: Vec<Project>, repos: Vec<Repo>) -> Result<()> {
    let milestones = config.zoho_projects[0].milestones.join("-");
    let path = format!("release-{}.md", milestones);
    let mut file = File::create(&path)?;

    let project_data = format_projects(projects, config)?;
    let repo_data = format_repos(repos, config);

    file.write_fmt(format_args!(
        "# Release {}\n\n{}{}\n",
        milestones, project_data, repo_data,
    ))?;

    Command::new("pandoc")
        .arg("-f")
        .arg("markdown")
        .arg("-t")
        .arg("pdf")
        .arg("-V")
        .arg("margin-top=3")
        .arg("-V")
        .arg("margin-left=3")
        .arg("-V")
        .arg("margin-right=3")
        .arg("-V")
        .arg("margin-bottom=3")
        .arg("--pdf-engine")
        .arg("wkhtmltopdf")
        .arg("--pdf-engine-opt")
        .arg("--enable-local-file-access")
        .arg("--css")
        .arg("styles/pdf.css")
        .arg("-o")
        .arg(format!("release-{}.pdf", milestones))
        .arg(path)
        .output()
        .expect("Failed to execute pandoc");
    Ok(())
}

fn run() -> Result<i32> {
    let config = config::parse_config("./config.toml");
    let repos = config.repos.clone();
    let projects = config.zoho_projects.clone();

    write_output(&config, projects, repos)?;

    Ok(0)
}

fn main() {
    pretty_env_logger::init();

    ::std::process::exit(match run() {
        Ok(_) => {
            println!("Goodbye");
            0
        }
        Err(err) => {
            eprintln!("Error occurred while running: {:?}", err);
            1
        }
    });
}
