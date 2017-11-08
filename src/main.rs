#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
extern crate reqwest;
extern crate chrono;
extern crate hyper;

mod errors;
mod pull_list;

use errors::*;
use pull_list::print_repos;
use dotenv::dotenv;

fn run() -> Result<i32> {
    dotenv().ok();
    print_repos()?;
    Ok(0)
}

quick_main!(run);
