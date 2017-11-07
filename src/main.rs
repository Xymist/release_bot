#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate dotenv;
extern crate reqwest;
extern crate chrono;
extern crate hyper;

mod errors;
mod pull_list;

use errors::*;
use pull_list::{Predicate, print_pulls_for_repo, app};
use dotenv::dotenv;

fn run() -> Result<()> {
    dotenv().ok();

    let args = app().get_matches();
    let pred = Predicate::from_args(&args)?;

    let repos: &[&str] = &["niciliketo/auction", "niciliketo/auction-frontend"];

    for repo in repos {
        print_pulls_for_repo(&repo, &pred)?;
    }

    Ok(())
}

quick_main!(run);
