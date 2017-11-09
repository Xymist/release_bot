use reqwest;
use errors::*;
use pull_list::release::Release;
use pull_list::predicate::Predicate;
use pull_list::pull::Pull;
use pull_list::pr_iterator::PRIterator;
use hyper::header::Authorization;
use std::{env, fmt};

#[derive(Debug)]
pub struct Repo {
    pub name: String,
    pub base: String,
    pub last_release: Option<Release>,
    pub pulls: Option<Vec<Pull>>,
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\n## {}\n\n###Last Release: {}\n",
            self.name,
            self.last_release.as_ref().unwrap()
        )
    }
}

impl Repo {
    pub fn construct(&mut self) -> Result<()> {
        self.populate_last_release()?;
        self.populate_pulls()?;

        Ok(())
    }

    fn populate_last_release(&mut self) -> Result<()> {
        if self.last_release.is_some() {
            return Ok(());
        }
        let url = format!("https://api.github.com/repos/{}/releases/latest", self.name);
        let client = reqwest::Client::new();
        let mut req = client.get(&url);

        if let Some(ref token) = env::var("GITHUB_TOKEN").ok() {
            req.header(Authorization(format!("token {}", token)));
        }

        let mut response = req.send()?;

        self.last_release = match response.status() {
            reqwest::StatusCode::Ok => Some(response.json::<Release>()?),
            reqwest::StatusCode::NotFound => Some(Release::default()),
            _ => bail!("Server error: {:?}", response.status()),
        };

        Ok(())
    }

    fn populate_pulls(&mut self) -> Result<()> {
        if self.pulls.is_some() {
            return Ok(());
        }

        let pred = Predicate::from_release(&self.last_release.as_ref().unwrap())?;

        let url = format!(
            "https://api.github.com/repos/{}/pulls?state=closed&base={}",
            self.name,
            self.base
        );

        let mut pull_iter = PRIterator::for_addr(&url)?
            .filter_map(Result::ok)
            .filter(|pull| pred.test(pull))
            .peekable();

        if pull_iter.peek().is_none() {
            return Ok(());
        }

        let pulls: Vec<Pull> = pull_iter.map(|pull| pull.add_tickets()).collect();

        self.pulls = Some(pulls);

        Ok(())
    }
}