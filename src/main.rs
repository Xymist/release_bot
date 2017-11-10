#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
extern crate reqwest;
extern crate chrono;
extern crate hyper;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate inflector;

mod errors;
mod pull_list;
mod zoho_bugs;

use errors::*;
use pull_list::print_repos;
use zoho_bugs::print_bugs;
use zoho_bugs::issue;
use dotenv::dotenv;

fn run() -> Result<i32> {
    dotenv().ok();
    // print_repos()?;
    let issues = issue::build_list(String::from("11.2.6"), vec![String::from("MD8196")])?;
    print_bugs(issues)?;
    Ok(0)
}

quick_main!(run);
