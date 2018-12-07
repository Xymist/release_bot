//! This crate is a documentation generation crate for single releases of Market Dojo; it accesses both
//! GitHub and the Zoho Projects API to retrieve data.

extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate hyper;
extern crate inflector;
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

use crate::config::{Config, Project};
use crate::errors::*;
use crate::pull_list::{csv_repo, format_repo, repo::Repo};
use crate::zoho_bugs::{
    classify_actions, issue, merge_actions, task, write_actions_csv, write_actions_md, zh_client,
};
use std::env;
use std::fs::File;
use std::io::prelude::*;

// Preamble for generated email or document; this might be removed in future as it is too specific
// to a single function of the crate.
const PREAMBLE: &str = "Hi everyone,\n
We have released a new version of Market Dojo to live.\n
Please let your customers know if they are listed and you feel the fixes will be relevant to them.
A complete list of changes is attached.\n
Many thanks to the whole team who have worked incredibly hard to make this release possible.\n";

fn format_preamble(config: &Config) -> String {
    let mut output: String = "".to_owned();
    let milestone_list: Vec<String> = config
        .zoho_projects
        .iter()
        .map(|p| p.milestones.join(", "))
        .collect();
    let milestones = milestone_list.join(", ");

    output.push_str(PREAMBLE);
    output.push_str(&format!(
        "\nThis includes development of the {} milestone(s).\n",
        milestones
    ));
    output
}

fn format_projects_as_md(projects: Vec<Project>, config: &Config) -> Result<String> {
    let mut output: String = "".to_owned();
    for project in projects {
        output.push_str(&format!(
            "\n## Closed Tickets and Tasks for {}\n\n",
            project.name
        ));

        let client = zh_client(project.id.parse::<i64>()?, config)?;
        let tasks = task::build_list(&client, &project.milestones)?;
        let issues = issue::build_list(&client, &project.milestones)?;
        output.push_str(&write_actions_md(merge_actions(
            classify_actions(issues),
            classify_actions(tasks),
        )));
    }
    Ok(output)
}

fn format_repos_as_md(repos: Vec<Repo>) -> String {
    let mut output: String = "".to_owned();
    for repo in repos {
        output.push_str(&format_repo(repo));
    }
    output
}

fn format_repos_as_csv(repos: Vec<Repo>) -> String {
    let mut output: String = "Repository,Pull Request,Contributor".to_owned();
    for repo in repos {
        if let Some(csv) = csv_repo(repo) {
            output.push_str(&format!("\n{}", csv));
        }
    }
    output
}

fn format_projects_as_csv(projects: Vec<Project>, config: &Config) -> Result<String> {
    let mut output: String = "Ticket Name,Ticket Type,Raised By,Clients".to_owned();
    for project in projects {
        let client = zh_client(project.id.parse::<i64>()?, config)?;
        let issues = issue::build_list(&client, &project.milestones)?;
        let tasks = task::build_list(&client, &project.milestones)?;
        output.push_str(
            &write_actions_csv(merge_actions(
                classify_actions(issues),
                classify_actions(tasks),
            ))
            .to_string(),
        );
    }
    Ok(output)
}

fn markdown_output(config: &Config, projects: Vec<Project>, repos: Vec<Repo>) -> Result<()> {
    let mut file = File::create(&format!(
        "release-{}.md",
        config.zoho_projects[0].milestones[0]
    ))?;
    file.write_fmt(format_args!(
        "# Release {}\n\n",
        config.zoho_projects[0].milestones[0]
    ))?;
    file.write_fmt(format_args!("{}", format_preamble(config)))?;
    file.write_fmt(format_args!("{}", format_projects_as_md(projects, config)?))?;
    file.write_fmt(format_args!("{}", format_repos_as_md(repos)))?;
    Ok(())
}

fn csv_output(config: &Config, projects: Vec<Project>, repos: Vec<Repo>) -> Result<()> {
    let mut project_file = File::create(&format!(
        "projects-{}.csv",
        config.zoho_projects[0].milestones[0]
    ))?;
    let mut repository_file = File::create(&format!(
        "repos-{}.csv",
        config.zoho_projects[0].milestones[0]
    ))?;
    project_file.write_fmt(format_args!(
        "{}",
        format_projects_as_csv(projects, config)?
    ))?;
    repository_file.write_fmt(format_args!("{}", format_repos_as_csv(repos)))?;
    Ok(())
}

fn run() -> Result<i32> {
    let args: Vec<String> = env::args().collect();
    let config = Config::default();
    let repos = config.repos.clone();
    let projects = config.zoho_projects.clone();

    let output_option = match args.len() {
        1 => "md",
        2 => &args[1],
        _ => bail!("Too many arguments"),
    };

    match output_option {
        "all" => {
            csv_output(&config, projects.clone(), repos.clone())?;
            markdown_output(&config, projects, repos)?
        }
        "csv" => csv_output(&config, projects, repos)?,
        "md" => markdown_output(&config, projects, repos)?,
        _ => println!("Not a valid output format. Try 'csv', 'md' or 'all'"),
    };

    Ok(0)
}

quick_main!(run);
