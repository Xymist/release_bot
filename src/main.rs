#![deny(
    unused_imports,
    rust_2018_idioms,
    rust_2018_compatibility,
    unsafe_code,
    clippy::all
)]

//! This crate is a documentation generation crate for single releases of Market Dojo.

mod regex;

use clap::Parser;
use color_eyre::{eyre::eyre, Report, Result};
use octocrab::{models::issues::Issue, Octocrab};
use regex::{client_details, feature_regexp, module_details};
use std::{
    collections::HashMap,
    fs::{DirBuilder, File},
    io::Write,
    process::Command,
    vec::IntoIter,
};
use tokio::sync::OnceCell;
use tracing::{error, info};

static CLIENT: OnceCell<Octocrab> = OnceCell::const_new();
fn client() -> &'static Octocrab {
    CLIENT.get().expect("Client not initialized")
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[clap(short, long)]
    milestone: String,
    #[clap(short, long, env = "GITHUB_TOKEN")]
    token: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let _ = CLIENT
        .get_or_try_init(|| async {
            Ok::<Octocrab, Report>(Octocrab::builder().personal_token(args.token).build()?)
        })
        .await;

    ::std::process::exit(match run(args.milestone.as_ref()).await {
        Ok(_) => {
            info!("Goodbye");
            0
        }
        Err(err) => {
            error!("Error occurred while running: {:?}", err);
            1
        }
    });
}

fn duration_to_string(duration: chrono::Duration) -> String {
    let mut components = Vec::new();
    let years = duration.num_days() / 365;
    let months = (duration.num_days() % 365) / 30;
    let weeks = (duration.num_days() % 30) / 7;
    let days = duration.num_days() % 7;
    let hours = duration.num_hours() % 24;

    if years > 0 {
        components.push(format!("{} years", years));
    }

    if months > 0 {
        components.push(format!("{} months", months));
    }

    if weeks > 0 {
        components.push(format!("{} weeks", weeks));
    }

    if days > 0 {
        components.push(format!("{} days", days));
    }

    if hours > 0 {
        components.push(format!("{} hours", hours));
    }

    components.join(", ")
}

#[derive(Default)]
struct ModuleStat {
    bugs: usize,
    features: usize,
}

enum OutputType {
    Latex,
    Markdown,
}

#[derive(Clone, Default)]
struct IssueData {
    client_requests: Vec<(u64, String, String)>,
    features: Vec<(u64, String, String)>,
    bugfixes: Vec<(u64, String, String)>,
    average_lifetime: String,
    module_stats: Vec<(String, usize, usize)>,
}

impl IssueData {
    fn client_requests(&self, output_type: OutputType) -> String {
        match output_type {
            OutputType::Latex => self
                .client_requests
                .iter()
                .map(|cr| format!("{} & {} & {}", cr.0, cr.1, cr.2))
                .collect::<Vec<String>>()
                .join(" \\\\\n"),
            OutputType::Markdown => self
                .client_requests
                .iter()
                .map(|cr| format!("| {} | {} | {} |", cr.0, cr.1, cr.2))
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }

    fn features(&self, output_type: OutputType) -> String {
        match output_type {
            OutputType::Latex => self
                .features
                .iter()
                .map(|f| format!("{} & {} & {}", f.0, f.1, f.2))
                .collect::<Vec<String>>()
                .join(" \\\\\n"),
            OutputType::Markdown => self
                .features
                .iter()
                .map(|f| format!("| {} | {} | {} |", f.0, f.1, f.2))
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }

    fn bugfixes(&self, output_type: OutputType) -> String {
        match output_type {
            OutputType::Latex => self
                .bugfixes
                .iter()
                .map(|b| format!("{} & {} & {}", b.0, b.1, b.2))
                .collect::<Vec<String>>()
                .join(" \\\\\n"),
            OutputType::Markdown => self
                .bugfixes
                .iter()
                .map(|b| format!("| {} | {} | {} |", b.0, b.1, b.2))
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }

    fn module_stats(&self, output_type: OutputType) -> String {
        match output_type {
            OutputType::Latex => self
                .module_stats
                .iter()
                .map(|m| {
                    if m.0.as_str() == "Total" {
                        format!(
                            "\\textbf{{{}}} & \\textbf{{{}}} & \\textbf{{{}}} & \\textbf{{{}}}",
                            m.0,
                            m.1,
                            m.2,
                            m.1 + m.2
                        )
                    } else {
                        format!("{} & {} & {} & {}", m.0, m.1, m.2, m.1 + m.2)
                    }
                })
                .collect::<Vec<String>>()
                .join(" \\\\\n"),
            OutputType::Markdown => self
                .module_stats
                .iter()
                .map(|m| {
                    if m.0.as_str() == "Total" {
                        format!(
                            "| **{}** | **{}** | **{}** | **{}** |",
                            m.0,
                            m.1,
                            m.2,
                            m.1 + m.2
                        )
                    } else {
                        format!("| {} | {} | {} | {} |", m.0, m.1, m.2, m.1 + m.2)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }
}

async fn fetch_issues(version: &str, repo: &str) -> Result<IssueData> {
    let issues = client()
        .search()
        .issues_and_pull_requests(&format!(
            "milestone:{} repo:marketdojo/{} is:closed is:issue",
            version, repo
        ))
        .send()
        .await?
        .into_iter();

    let mut client_requests = Vec::new();
    let mut features = Vec::new();
    let mut bugfixes = Vec::new();
    let mut module_stats = HashMap::new();
    let average_lifetime = average_lifetime(issues.clone())?;

    for issue in issues {
        let title = title(&issue);
        let body = body(&issue);
        let client_details = client_details(&body).await;
        let modules = module_details(&body).await;
        let feature = feature_regexp().await?.is_match(&title);

        if let Some(modules) = modules {
            // In debug env, print the modules for each issue
            if cfg!(debug_assertions) {
                println!("{}: {}", title, modules.join(", "));
            }

            for module in modules {
                let stat = module_stats
                    .entry(module.to_string())
                    .or_insert(ModuleStat {
                        bugs: 0,
                        features: 0,
                    });

                if feature {
                    stat.features += 1;
                } else {
                    stat.bugs += 1;
                }
            }
        }

        if let Some(details) = client_details {
            client_requests.push((issue.number, title, details));
        } else if feature {
            features.push((issue.number, title, issue.user.login));
        } else {
            bugfixes.push((issue.number, title, issue.user.login));
        }
    }

    let mut module_stats: Vec<(String, usize, usize)> = module_stats
        .iter()
        .map(|(module, count)| (module.clone(), count.features, count.bugs))
        .collect();
    module_stats.sort_by(|a, b| a.0.cmp(&b.0));

    let feature_count = module_stats.iter().map(|(_, f, _)| f).sum();
    info!("Feature count: {}", feature_count);

    let bug_count = module_stats.iter().map(|(_, _, b)| b).sum();
    info!("Bug count: {}", bug_count);

    module_stats.push(("Total".to_string(), feature_count, bug_count));

    Ok(IssueData {
        client_requests,
        features,
        bugfixes,
        average_lifetime,
        module_stats,
    })
}

fn average_lifetime(issues: IntoIter<octocrab::models::issues::Issue>) -> Result<String> {
    let len = issues.len() as i64;

    let average_lifetime = issues
        .filter_map(|issue| {
            issue.closed_at.and_then(|closed_at| {
                closed_at
                    .timestamp()
                    .checked_sub(issue.created_at.timestamp())
            })
        })
        .sum::<i64>()
        .checked_div(len)
        .ok_or(eyre!("No issues found for this milestone"))?;

    Ok(duration_to_string(chrono::Duration::seconds(
        average_lifetime,
    )))
}

fn title(issue: &Issue) -> String {
    issue
        .title
        .trim()
        .replace('_', "\\_")
        .replace('&', "\\&")
        .replace('#', "\\#")
}

fn body(issue: &Issue) -> String {
    issue
        .body
        .as_ref()
        .map(|body| body.trim().replace("\r\n", "\n"))
        .unwrap_or_default()
}

#[derive(Clone, Default)]
struct PrStats {
    total_count: usize,
    average_lifetime: String,
    contributor_count: usize,
}

async fn pr_stats(version: &str, repo: &str) -> Result<PrStats> {
    let pulls = client()
        .search()
        .issues_and_pull_requests(&format!(
            "milestone:{} repo:marketdojo/{} is:closed is:pr",
            version, repo
        ))
        .send()
        .await?
        .into_iter();
    let len = pulls.len();

    let mut stats = PrStats {
        total_count: len,
        average_lifetime: "".to_string(),
        contributor_count: pulls
            .clone()
            .map(|pr| pr.user.login)
            .collect::<std::collections::HashSet<_>>()
            .len(),
    };

    let average_lifetime = pulls
        .filter_map(|pr| {
            pr.closed_at
                .and_then(|closed_at| closed_at.timestamp().checked_sub(pr.created_at.timestamp()))
        })
        .sum::<i64>()
        .checked_div(len as i64)
        .ok_or(eyre!("No PRs found for this milestone"))?;

    stats.average_lifetime = duration_to_string(chrono::Duration::seconds(average_lifetime));

    Ok(stats)
}

async fn construct_latex_report(version: &str, issues: &IssueData, pull_stats: &PrStats) -> String {
    std::fmt::format(format_args!(
        include_str!("../resources/report_format.tex.tmpl"),
        version = version,
        n_prs = pull_stats.total_count,
        n_closed = issues.client_requests.len() + issues.features.len() + issues.bugfixes.len(),
        client_request_table = issues.client_requests(OutputType::Latex),
        feature_table = issues.features(OutputType::Latex),
        bugfix_table = issues.bugfixes(OutputType::Latex),
        avg_lifetime = issues.average_lifetime,
        avg_pr_lifetime = pull_stats.average_lifetime,
        module_table = issues.module_stats(OutputType::Latex),
        n_contributors = pull_stats.contributor_count,
    ))
}

async fn construct_markdown_report(
    version: &str,
    issues: &IssueData,
    pull_stats: &PrStats,
) -> String {
    std::fmt::format(format_args!(
        include_str!("../resources/report_format.md.tmpl"),
        release_date = chrono::Utc::now().format("%Y-%m-%d"),
        version = version,
        n_prs = pull_stats.total_count,
        n_closed = issues.client_requests.len() + issues.features.len() + issues.bugfixes.len(),
        client_request_table = issues.client_requests(OutputType::Markdown),
        feature_table = issues.features(OutputType::Markdown),
        bugfix_table = issues.bugfixes(OutputType::Markdown),
        avg_lifetime = issues.average_lifetime,
        avg_pr_lifetime = pull_stats.average_lifetime,
        module_table = issues.module_stats(OutputType::Markdown),
        n_contributors = pull_stats.contributor_count,
    ))
}

// tectonic <input> --outfmt <format> --chatter <level> --pass <pass> --format <path> --color <when>
async fn generate_pdf(path: &str) -> Result<()> {
    let dir_path = "resources";
    DirBuilder::new().recursive(true).create(dir_path)?;
    let logo = File::create_new("resources/mdlogo.png");

    if let Ok(mut logo) = logo {
        info!("Creating logo");
        logo.write_all(include_bytes!("../resources/mdlogo.png"))?;
    }

    let output = Command::new("tectonic")
        .arg(path)
        .arg("--outfmt")
        .arg("pdf")
        .output()?;

    if !output.status.success() {
        return Err(eyre!(
            "Failed to generate PDF, with the following stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

async fn run(version: &str) -> Result<i32> {
    info!("Fetching issues");
    let issues = fetch_issues(version, "auction").await?;

    info!("Fetching PR stats");
    let pull_stats = pr_stats(version, "auction").await?;

    latex_report(version, &issues, &pull_stats).await?;
    info!("Generated LaTeX and PDF reports");

    markdown_report(version, &issues, &pull_stats).await?;
    info!("Generated Markdown report");

    Ok(0)
}

async fn latex_report(version: &str, issues: &IssueData, pull_stats: &PrStats) -> Result<()> {
    let dir_path = "releases";
    DirBuilder::new().recursive(true).create(dir_path)?;
    let path = format!("releases/release-{}.tex", version);
    let mut file = File::create(&path)?;

    file.write_all(
        construct_latex_report(version, issues, pull_stats)
            .await
            .as_bytes(),
    )?;

    generate_pdf(&path).await
}

async fn markdown_report(version: &str, issues: &IssueData, pull_stats: &PrStats) -> Result<()> {
    let dir_path = "releases";
    DirBuilder::new().recursive(true).create(dir_path)?;
    let path = format!("releases/release-{}.md", version);
    let mut file = File::create(&path)?;

    file.write_all(
        construct_markdown_report(version, issues, pull_stats)
            .await
            .as_bytes(),
    )?;

    Ok(())
}
