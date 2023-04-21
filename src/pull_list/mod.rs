mod pr_iterator;
mod predicate;
mod pull;
mod release;
pub mod repo;

use crate::config::Config;
use crate::pull_list::pull::Pull;
use crate::pull_list::repo::Repo;
use inflector::Inflector;
use std::collections::HashMap;
use std::fmt::Write;

// TODO: This should take a parameter repo list so that Main can use it
// properly.
pub fn format_repo(mut repo: Repo, config: &Config) -> String {
    let mut output: String = "".to_owned();

    if repo.construct(config).is_err() {
        return output;
    };

    output = format!("{}", repo);

    if repo.pulls.is_some() {
        output.push_str("\n| Pull Request | Contributor |\n| --- | --- |");

        let sorted_contributors = extract_and_sort_contributors(repo);

        for (contributor, cont_pulls) in sorted_contributors {
            // This .rev() corrects the GitHub default order and gives us
            // a chronological record
            for cont_pull in cont_pulls.into_iter().rev() {
                write!(output, "\n| {} | {} |", cont_pull, contributor).unwrap();
            }
        }
    }
    output
}

fn extract_and_sort_contributors(repo: Repo) -> Vec<(String, Vec<Pull>)> {
    // Initialize a hashmap to contain the list of PRs for each
    // contributor; thanks to GitHub's defaults they will be
    // in 'newest first' order, which we will need to correct for
    // later on.
    let mut contributors: HashMap<String, Vec<Pull>> = HashMap::new();
    for pull in repo.pulls.expect("No pulls found for repo!") {
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
