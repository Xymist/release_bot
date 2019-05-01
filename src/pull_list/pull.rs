use chrono::{offset, DateTime};
use serde_derive::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug, Clone)]
pub struct GithubUser {
    pub id: u32,
    pub login: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pull {
    html_url: String,
    title: String,
    pub user: GithubUser,
    pub closed_at: DateTime<offset::Utc>,
}

impl fmt::Display for Pull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]({})", self.title, self.html_url)
    }
}

