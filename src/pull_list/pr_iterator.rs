use reqwest;
use errors::*;
use std::env;
use hyper::header::{Authorization, Link, RelationType};
use serde::de::DeserializeOwned;

pub struct PRIterator<T> {
    pub items: <Vec<T> as IntoIterator>::IntoIter,
    pub next_link: Option<String>,
    pub client: reqwest::Client,
    pub github_token: Option<String>,
}

impl<T> PRIterator<T>
where
    T: DeserializeOwned,
{
    pub fn for_addr(url: &str) -> Result<Self> {
        Ok(PRIterator {
            items: Vec::new().into_iter(),
            next_link: Some(url.to_owned()),
            client: reqwest::Client::new(),
            github_token: env::var("GITHUB_TOKEN").ok(),
        })
    }

    fn try_next(&mut self) -> Result<Option<T>> {
        if let Some(pull) = self.items.next() {
            return Ok(Some(pull));
        }

        if self.next_link.is_none() {
            return Ok(None);
        }

        let url = self.next_link.take().unwrap();
        let mut req = self.client.get(&url);

        if let Some(ref token) = self.github_token {
            req.header(Authorization(format!("token {}", token)));
        }

        let mut response = req.send()?;
        if !response.status().is_success() {
            bail!("Server error: {:?}", response.status());
        }

        self.items = response.json::<Vec<T>>()?.into_iter();

        // The response that GitHub's API will give is limited to a few PRs;
        // a header is attached with the url of the next set.
        if let Some(header) = response.headers().get::<Link>() {
            for val in header.values() {
                if val.rel()
                    .map(|rel| rel.contains(&RelationType::Next))
                    .unwrap_or(false)
                {
                    self.next_link = Some(val.link().to_owned());
                    break;
                }
            }
        }

        Ok(self.items.next())
    }
}

impl<T> Iterator for PRIterator<T>
where
    T: DeserializeOwned,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(val)) => Some(Ok(val)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}