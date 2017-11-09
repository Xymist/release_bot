use chrono::{DateTime, offset};
use std::fmt;
use pull_list::user::User;
use regex::Regex;

#[derive(Deserialize, Debug)]
pub struct Pull {
    html_url: String,
    title: String,
    pub user: User,
    pub bug_tickets: Option<Vec<String>>,
    pub closed_at: DateTime<offset::Utc>,
}

impl fmt::Display for Pull {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "- [{}]({})", self.title, self.html_url)
    }
}

impl Pull {
    pub fn add_tickets(mut self) -> Pull {
        self.bug_tickets = self.parse_tickets();
        self
    }

    fn parse_tickets(&self) -> Option<Vec<String>> {
        lazy_static! {
            static ref TKS: Regex = Regex::new(r"^\[(#(MD|CD|QQ)?\d+(, )?)+\]").unwrap();
            static ref TK: Regex = Regex::new(r"#((MD|CD|QQ)?\d+)").unwrap();
        }

        // First we get the entire [#MD1234, #MD5678] section
        let tk_tag_list = match TKS.captures(&self.title) {
            Some(tag_list) => tag_list,
            None => return None,
        };

        let t_iter = TK.captures_iter(&tk_tag_list[0]);
        let tags: Vec<String> = t_iter.map(|tk_tag| format!("{}", &tk_tag[0])).collect();
        return Some(tags);
    }
}