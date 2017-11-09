mod pr_iterator;
mod predicate;
mod release;
mod repo;
mod pull;
mod user;

use errors::*;
use self::repo::Repo;

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

pub fn print_repos() -> Result<()> {
    for mut repo in repo_list().into_iter() {
        repo.construct()?;
        if repo.pulls.is_some() {
            println!("{}", repo);
            for pull in repo.pulls.unwrap() {
                println!("{}", pull);
            }
        }
    }

    Ok(())
}