use reqwest;
use errors::*;
use std::{env, fmt};
use chrono::{DateTime, NaiveDate, Utc, TimeZone, offset};
use hyper::header::{Authorization, Link, RelationType};
use serde::de::DeserializeOwned;

fn repo_list() -> Vec<Repo> {
    return vec![
        Repo {
            name: String::from("niciliketo/auction-frontend"),
            base: String::from("master"),
        },
        Repo {
            name: String::from("niciliketo/auction"),
            base: String::from("development"),
        },
    ];
}

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

#[derive(Deserialize, Debug)]
struct Release {
    // id: u32,
    // name: String,
    // tag_name: String,
    // body: String,
    created_at: DateTime<offset::Utc>,
}

// impl fmt::Display for Release {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "- {} {}, {}", self.name, self.body, self.created_at)
//     }
// }

#[derive(Debug)]
struct Repo {
    name: String,
    base: String,
}

impl Repo {
    fn most_recent_release(&self) -> Result<Release> {
        let url = format!("https://api.github.com/repos/{}/releases/latest", self.name);
        let client = reqwest::Client::new();
        let mut req = client.get(&url);

        if let Some(ref token) = env::var("GITHUB_TOKEN").ok() {
            req.header(Authorization(format!("token {}", token)));
        }

        let mut response = req.send()?;

        match response.status() {
            reqwest::StatusCode::Ok => Ok(response.json::<Release>()?),
            reqwest::StatusCode::NotFound => Ok(Release {
                created_at: Utc.ymd(2000, 01, 01).and_hms(0, 0, 0),
            }),
            _ => bail!("Server error: {:?}", response.status()),
        }
    }
}

#[derive(Deserialize, Debug)]
struct Pull {
    html_url: String,
    title: String,
    user: User,
    closed_at: DateTime<offset::Utc>,
}

impl fmt::Display for Pull {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "- @{} [{}]({})",
            self.user.login,
            self.title,
            self.html_url
        )
    }
}

struct PRIterator<T> {
    items: <Vec<T> as IntoIterator>::IntoIter,
    next_link: Option<String>,
    client: reqwest::Client,
    github_token: Option<String>,
}

impl<T> PRIterator<T>
where
    T: DeserializeOwned,
{
    fn for_addr(url: &str) -> Result<Self> {
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

struct Predicate {
    since: Option<NaiveDate>,
}

impl Predicate {
    fn from_release<'a>(release: &Release) -> Result<Predicate> {
        Ok(Predicate {
            since: Some(release.created_at.date().naive_utc()),
        })
    }

    fn test(&self, pull: &Pull) -> bool {
        let pull_closed = pull.closed_at.date().naive_utc();
        self.since.map(|v| pull_closed > v).unwrap_or(true)
    }
}

fn print_pulls_for_repo(repo: &Repo, pred: &Predicate) -> Result<()> {

    let url = format!(
        "https://api.github.com/repos/{}/pulls?state=closed&base={}",
        repo.name,
        repo.base
    );

    let mut pulls = PRIterator::for_addr(&url)?
        .filter_map(Result::ok)
        .filter(|pull| pred.test(pull))
        .peekable();

    if pulls.peek().is_none() {
        return Ok(());
    }

    println!("\n#### {} ####\n", repo.name);

    for pull in pulls {
        println!("{}", pull);
    }

    Ok(())
}

pub fn print_repos() -> Result<()> {
    for repo in repo_list().into_iter() {
        let last_release = match repo.most_recent_release() {
            Ok(release) => Some(release),
            Err(_) => None,
        };
        if last_release.is_some() {
            let pred = Predicate::from_release(&last_release.unwrap())?;
            print_pulls_for_repo(&repo, &pred)?;
        }
    }

    Ok(())
}