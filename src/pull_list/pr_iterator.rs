use crate::errors::*;
use crate::pull_list::predicate::Predicate;
use crate::pull_list::pull::Pull;
use crate::Config;
use reqwest;

pub struct PRIterator {
    pub items: <Vec<Pull> as IntoIterator>::IntoIter,
    pub next_link: Option<String>,
    pub client: reqwest::Client,
    pub github_token: String,
    pub predicate: Option<Predicate>,
}

impl PRIterator {
    pub fn for_addr(url: &str, pred: Option<Predicate>, config: &Config) -> Result<Self> {
        Ok(PRIterator {
            items: Vec::new().into_iter(),
            next_link: Some(url.to_owned()),
            client: reqwest::Client::new(),
            github_token: config.github_token.clone(),
            predicate: pred,
        })
    }

    fn try_next(&mut self) -> Result<Option<Pull>> {
        if let Some(pull) = self.items.next() {
            return Ok(Some(pull));
        }

        if self.next_link.is_none() {
            return Ok(None);
        }

        let url = self.next_link.take().unwrap();
        let mut req = self.client.get(&url);

        req = req.header(
            reqwest::header::AUTHORIZATION,
            format!("token {}", &self.github_token),
        );

        let mut response = req.send()?;
        if !response.status().is_success() {
            panic!("Server error: {:?}", response.status());
        }

        let returned_items = response.json::<Vec<Pull>>()?;

        // We only bother getting the next set if the one we just processed
        // appears not to be the end of the collection.
        if returned_items.len() == 100 {
            // The response that GitHub's API will give is limited to a few PRs;
            // a header is attached with the url of the next set.
            let next_link: &str = response
                .headers()
                .get_all(reqwest::header::LINK)
                .iter()
                .map(reqwest::header::HeaderValue::to_str)
                .map(std::result::Result::unwrap)
                .filter(|rel| rel.contains("rel=\"next\""))
                .collect::<Vec<&str>>()
                .first()
                .unwrap();

            self.next_link = Some(next_link.to_owned());
        }

        let item_iter = returned_items.into_iter();

        self.items = if let Some(ref pred) = self.predicate {
            item_iter
                .filter(|pull| pred.test(pull))
                .collect::<Vec<Pull>>()
                .into_iter()
        } else {
            item_iter
        };

        Ok(self.items.next())
    }
}

impl Iterator for PRIterator {
    type Item = Result<Pull>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(val)) => Some(Ok(val)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}
