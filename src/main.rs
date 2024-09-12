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
use color_eyre::{Report, Result};
use octocrab::Octocrab;
use regex::{client_regexp, feature_regexp, module_regexp};
use std::{collections::HashMap, io::Write, process::Command};
use tokio::sync::OnceCell;
use tracing::error;

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
            println!("Goodbye");
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
struct IssueData {
    client_requests: Vec<String>,
    features: Vec<String>,
    bugfixes: Vec<String>,
    total_count: usize,
    average_lifetime: String,
    #[allow(dead_code)]
    module_stats: HashMap<String, i64>,
}

async fn fetch_issues(milestone: &str, repo: &str) -> Result<IssueData> {
    let issues = client()
        .search()
        .issues_and_pull_requests(&format!(
            "milestone:{} repo:marketdojo/{} is:closed is:issue",
            milestone, repo
        ))
        .send()
        .await?
        .into_iter();

    let mut client_items = Vec::new();
    let mut feature_items = Vec::new();
    let mut bugfix_items = Vec::new();
    let total_count = issues.len();
    let mut module_stats = HashMap::new();
    let average_lifetime = issues
        .clone()
        .filter_map(|issue| {
            issue.closed_at.and_then(|closed_at| {
                closed_at
                    .timestamp()
                    .checked_sub(issue.created_at.timestamp())
            })
        })
        .sum::<i64>()
        / total_count as i64;

    for issue in issues {
        let title = issue.title.clone();
        let body = issue.body.unwrap_or_default();
        let client_details = client_regexp().await?.captures(&body).and_then(|c| {
            c.get(2)
                .filter(|m| m.as_str() != "_No response_")
                .map(|m| m.as_str())
        });
        let module_details = module_regexp()
            .await?
            .captures(&body)
            .and_then(|c| {
                c.get(2)
                    .filter(|m| m.as_str() != "_No response_")
                    .map(|m| m.as_str())
            })
            .unwrap_or("Unknown");
        let feature = feature_regexp().await?.is_match(&title);
        let modules = module_details.split(", ");
        for module in modules {
            *module_stats.entry(module.to_string()).or_insert(0) += 1;
        }

        if let Some(details) = client_details {
            client_items.push(format!(
                "| #{} | {} | {} | {} | {} |",
                issue.number, issue.title, issue.user.login, module_details, details
            ));
        } else if feature {
            feature_items.push(format!(
                "| #{} | {} | {} | {} |",
                issue.number, issue.title, issue.user.login, module_details,
            ));
        } else {
            bugfix_items.push(format!(
                "| #{} | {} | {} | {} |",
                issue.number, issue.title, issue.user.login, module_details,
            ));
        }
    }

    Ok(IssueData {
        client_requests: client_items,
        features: feature_items,
        bugfixes: bugfix_items,
        average_lifetime: duration_to_string(chrono::Duration::seconds(average_lifetime)),
        total_count,
        module_stats,
    })
}

#[derive(Default)]
struct PrStats {
    total_count: usize,
    average_lifetime: String,
}

async fn pr_stats(milestone: &str, repo: &str) -> Result<PrStats> {
    let pulls = client()
        .search()
        .issues_and_pull_requests(&format!(
            "milestone:{} repo:marketdojo/{} is:closed is:pr",
            milestone, repo
        ))
        .send()
        .await?
        .into_iter();
    let len = pulls.len();

    let mut stats = PrStats {
        total_count: len,
        average_lifetime: "".to_string(),
    };

    let average_lifetime = pulls
        .filter_map(|pr| {
            pr.closed_at
                .and_then(|closed_at| closed_at.timestamp().checked_sub(pr.created_at.timestamp()))
        })
        .sum::<i64>()
        / len as i64;

    stats.average_lifetime = duration_to_string(chrono::Duration::seconds(average_lifetime));

    Ok(stats)
}

async fn construct_report(version: &str, release_date: &str) -> String {
    let issues = fetch_issues(version, "auction").await.unwrap_or_default();
    let pr_stats = pr_stats(version, "auction").await.unwrap_or_default();

    std::fmt::format(format_args!(
        include_str!("report_format.md.tmpl"),
        version = version,
        release_date = release_date,
        n_tickets = issues.total_count,
        n_prs = pr_stats.total_count,
        n_features = issues.features.len(),
        n_bugfixes = issues.bugfixes.len(),
        client_request_table = issues.client_requests.join("\n"),
        feature_table = issues.features.join("\n"),
        bugfix_table = issues.bugfixes.join("\n"),
        avg_lifetime = issues.average_lifetime,
        avg_pr_lifetime = pr_stats.average_lifetime
    ))
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

async fn run(milestone: &str) -> Result<i32> {
    let path = format!("release-{}.md", milestone);
    let mut file = std::fs::File::create(&path)?;
    file.write_all(
        construct_report(
            milestone,
            chrono::Utc::now().date_naive().to_string().as_str(),
        )
        .await
        .as_bytes(),
    )?;
    generate_pdf(milestone, &path)?;

    Ok(0)
}
