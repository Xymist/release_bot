mod github_user;
mod pr_iterator;
mod predicate;
mod pull;
mod release;
pub mod repo;

use self::pull::Pull;
use self::repo::Repo;
use inflector::Inflector;
use std::collections::HashMap;

// TODO: This should take a parameter repo list so that Main can use it
// properly.
pub fn format_repo(repo: Repo) -> String {
    let mut output: String = format!("{}", repo);
    if repo.pulls.is_some() {
        output.push_str("\n| Pull Request | Contributor |\n| --- | --- |");

        let sorted_contributors = extract_and_sort_contributors(repo);

        for (contributor, cont_pulls) in sorted_contributors {
            // This .rev() corrects the GitHub default order and gives us
            // a chronological record
            for cont_pull in cont_pulls.into_iter().rev() {
                output.push_str(&format!("\n| {} | {} |", cont_pull, contributor));
            }
        }
    }
    output
}

pub fn csv_repo(repo: Repo) -> Option<String> {
    if repo.pulls.is_some() {
        let repo_name = repo.name.clone();
        let sorted_contributors = extract_and_sort_contributors(repo);
        let mut csv_lines: Vec<String> = vec![];
        for (contributor, cont_pulls) in sorted_contributors {
            // This .rev() corrects the GitHub default order and gives us
            // a chronological record
            for cont_pull in cont_pulls.into_iter().rev() {
                csv_lines.push(format!("{},{},{}", repo_name, cont_pull, contributor));
            }
        }
        let csv = csv_lines.join("\n");
        Some(csv)
    } else {
        None
    }
}

fn extract_and_sort_contributors(repo: Repo) -> Vec<(String, Vec<Pull>)> {
    // Initialize a hashmap to contain the list of PRs for each
    // contributor; thanks to GitHub's defaults they will be
    // in 'newest first' order, which we will need to correct for
    // later on.
    let mut contributors: HashMap<String, Vec<Pull>> = HashMap::new();
    for pull in repo.pulls.unwrap() {
        // TODO: Find something better than titlecase; things like CIS
        // need to be fully capitalized... might need something custom.
        // Or just ignore it, it's probable that nobody will complain.
        let contributor = pull.user.login.clone().to_title_case();
        let blank = vec![];
        let cont_pulls = contributors.entry(contributor).or_insert(blank);
        cont_pulls.push(pull)
    }

    // Convert the HashMap into something that can be sorted; most
    // frequent contributors at the bottom, least at the top
    let mut sortable: Vec<(String, Vec<Pull>)> = contributors.into_iter().collect();
    sortable.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
    sortable
}
