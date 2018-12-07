use crate::pull_list::github_user::GithubUser;
use chrono::{offset, DateTime};
use std::fmt;

#[derive(Deserialize, Debug, Clone)]
pub struct Pull {
    html_url: String,
    title: String,
    pub user: GithubUser,
    pub closed_at: DateTime<offset::Utc>,
}

impl fmt::Display for Pull {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]({})", self.title, self.html_url)
    }
}
