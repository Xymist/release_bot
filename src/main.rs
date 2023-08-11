#![warn(missing_docs)]
#![deny(
    unused_imports,
    rust_2018_idioms,
    rust_2018_compatibility,
    unsafe_code,
    clippy::all,
    dead_code
)]
#![feature(once_cell_try)]

//! This crate is a documentation generation crate for single releases of Market Dojo; it accesses both
//! GitHub and the Zoho Projects API to retrieve data.

mod config;
mod pull_list;
mod zoho_bugs;

use crate::config::{Config, Project};
use crate::pull_list::{format_repo, repo::Repo};
use crate::zoho_bugs::{classify_actions, issue, merge_actions, task, write_actions, zh_client};

use std::fmt::Write as _;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::sync::Arc;
use std::thread;

use tracing::error;

use color_eyre::{
    eyre::{eyre, WrapErr},
    Result,
};

fn format_projects(projects: Vec<Project>, config: &Config) -> Result<String> {
    let mut output: String = "".to_owned();
    for project in projects {
        write!(
            output,
            "\n## Closed Tickets and Tasks for {}\n\n",
            project.name
        )
        .wrap_err("Failed to write title text to output string")?;

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
            .map_err(|_| eyre!("Task list builder thread panicked"))??;
        let issues = issue_thread
            .join()
            .map_err(|_| eyre!("Issue list builder thread panicked"))??;

        output.push_str(&write_actions(merge_actions(
            classify_actions(tasks),
            classify_actions(issues),
        ))?);
    }

    Ok(output)
}

fn format_repos(repos: Vec<Repo>, config: &Config) -> Result<String> {
    let mut output: String = "".to_owned();
    let mut children = Vec::new();
    let sync_config = Arc::new(config.clone());

    for repo in repos.into_iter() {
        let conf = Arc::clone(&sync_config);
        children.push(thread::spawn(move || -> Result<String> {
            format_repo(repo, &conf)
        }));
    }

    for child in children {
        output.push_str(
            &child
                .join()
                .map_err(|_| eyre!("Repo formatting thread failed"))??,
        );
    }

    Ok(output)
}

fn write_output(config: &Config, projects: Vec<Project>, repos: Vec<Repo>) -> Result<()> {
    let milestones = config.zoho_projects[0].milestones.join("-");
    let path = format!("release-{}.md", milestones);
    let mut file = File::create(&path)?;

    let project_data = format_projects(projects, config)?;
    let repo_data = format_repos(repos, config)?;

    file.write_fmt(format_args!(
        "# Release {}\n\n{}{}\n",
        milestones, project_data, repo_data,
    ))?;

    generate_pdf(&milestones, &path)?;

    Ok(())
}

// Convert markdown to pdf
// Excuting pandoc with the following arguments:
// -f markdown: input format
// -t pdf: output format
// -V margin-top=3: set the top margin to 3cm
// -V margin-left=3: set the left margin to 3cm
// -V margin-right=3: set the right margin to 3cm
// -V margin-bottom=3: set the bottom margin to 3cm
// --pdf-engine wkhtmltopdf: use wkhtmltopdf as the pdf engine
// --pdf-engine-opt --enable-local-file-access: allow wkhtmltopdf to access local files
// -c styles/pdf.css: use the css file in the styles directory
// -o release-<milestones>.pdf: output to a file named release-<milestones>.pdf
// <path>: the path to the markdown file
fn generate_pdf(milestones: &str, path: &str) -> Result<()> {
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
        .arg("-c")
        .arg("styles/pdf.css")
        .arg("-o")
        .arg(format!("release-{}.pdf", milestones))
        .arg(path)
        .output()?;

    Ok(())
}

fn run() -> Result<i32> {
    let config = config::parse_config("./config.toml")?;
    let repos = config.repos.clone();
    let projects = config.zoho_projects.clone();

    write_output(&config, projects, repos)?;

    Ok(0)
}

fn main() {
    tracing_subscriber::fmt::init();

    ::std::process::exit(match run() {
        Ok(_) => {
            println!("Goodbye");
            0
        }
        Err(err) => {
            error!("Error occurred while running: {:?}", err);
            1
        }
    });
}
