mod pr_iterator;
mod predicate;
mod release;
mod repo;
mod pull;
mod user;

use errors::*;
use self::repo::Repo;
use self::pull::Pull;
use std::collections::HashMap;
use inflector::Inflector;

// TODO: This should be in a TOML config file (i.e. [[repo]]) parsed in Main.
// Hardcoding for now so I don't have to bother with a TOML parser upfront.
fn repo_list() -> Vec<Repo> {
    return vec![
        Repo {
            name: String::from("niciliketo/auction-frontend"),
            base: String::from("master"),
            last_release: None,
            pulls: None,
        },
        Repo {
            name: String::from("niciliketo/auction"),
            base: String::from("development"),
            last_release: None,
            pulls: None,
        },
    ];
}

// TODO: This should take a parameter repo list so that Main can use it
// properly.
pub fn print_repos() -> Result<()> {
    for mut repo in repo_list() {
        repo.construct()?;
        if repo.pulls.is_some() {
            println!("{}", repo);
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
                let cont_pulls = contributors.entry(contributor).or_insert(vec![]);
                cont_pulls.push(pull)
            }

            // Convert the HashMap into something that can be sorted; most
            // frequent contributors at the bottom, least at the top
            let mut sortable: Vec<(&String, &Vec<Pull>)> = contributors.iter().collect();
            sortable.sort_by(|a, b| a.1.len().cmp(&b.1.len()));

            for (contributor, cont_pulls) in sortable {
                println!("\n#### {} ({})", contributor, cont_pulls.len());
                // This .rev() corrects the GitHub default order and gives us
                // a chronological record
                for cont_pull in cont_pulls.into_iter().rev() {
                    println!("{}", cont_pull);
                }
            }
        }
    }

    Ok(())
}