use crate::pull_list::pr_iterator::PRIterator;
use crate::pull_list::predicate::Predicate;
use crate::pull_list::pull::Pull;
use crate::pull_list::release::Release;
use crate::Config;
use color_eyre::{eyre::eyre, Result};
use reqwest;
use serde_derive::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug, Clone)]
pub struct Repo {
    pub name: String,
    pub last_release: Option<Release>,
    pub pulls: Option<Vec<Pull>>,
    base: String,
    since: Option<String>,
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\n\n## Closed Pull Requests for {}\n\n### Last Release: {}",
            self.name,
            self.last_release.as_ref().ok_or(fmt::Error)?
        )
    }
}

impl Repo {
    pub fn construct(&mut self, config: &Config) -> Result<()> {
        self.populate_last_release(config)?;
        self.populate_pulls(config)?;

        Ok(())
    }

    fn populate_last_release(&mut self, config: &Config) -> Result<()> {
        if self.last_release.is_some() {
            return Ok(());
        }

        let tag = if let Some(ref last_release_tag) = self.since {
            format!("tags/{}", last_release_tag)
        } else {
            "latest".to_owned()
        };

        let url = format!(
            "https://api.github.com/repos/{}/releases/{}",
            self.name, tag
        );
        let client = reqwest::Client::new();
        let mut req = client.get(&url);

        req = req.header(
            reqwest::header::AUTHORIZATION,
            format!("token {}", config.github_token),
        );

        let mut response = req.send()?;

        self.last_release = match response.status() {
            reqwest::StatusCode::OK => Some(response.json::<Release>()?),
            reqwest::StatusCode::NOT_FOUND => Some(Release::default()),
            _ => panic!("Server error: {:?}", response.status()),
        };

        Ok(())
    }

    fn populate_pulls(&mut self, config: &Config) -> Result<()> {
        if self.pulls.is_some() {
            return Ok(());
        }

        let pred = Predicate::from_release(
            self.last_release
                .as_ref()
                .ok_or_else(|| eyre!("Repo has no last release; has it been initialized?"))?,
        )?;

        let base_url = format!(
            "https://api.github.com/repos/{}/pulls?state=closed&base={}&per_page=100",
            self.name, self.base
        );

        let mut base_pull_iter = PRIterator::for_addr(&base_url, Some(pred), config)?
            .filter_map(Result::ok)
            .peekable();

        if base_pull_iter.peek().is_none() {
            return Ok(());
        }

        let pulls: Vec<Pull> = base_pull_iter.collect();

        self.pulls = Some(pulls);

        Ok(())
    }
}
