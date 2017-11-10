use errors::*;
use std::{fmt, env};
use zoho_bugs::winnower::Winnower;
use reqwest;

#[derive(Deserialize, Debug)]
pub struct IssueList {
    pub milestone: Option<String>,
    pub issue_labels: Option<Vec<String>>,
    pub bugs: Option<Vec<Issue>>,
}

#[derive(Deserialize, Debug)]
pub struct Issue {
    pub key: Option<String>,
    pub title: Option<String>,
    pub milestone: Option<Milestone>,
    pub customfields: Option<Vec<CustomField>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomField {
    pub column_name: String,
    pub label_name: String,
    pub value: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Milestone {
    pub name: String,
    pub id: String,
}

impl Default for Milestone {
    fn default() -> Milestone {
        Milestone {
            name: String::from("No Milestone"),
            id: String::from("0"),
        }
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "- [{}] {}",
            self.key.as_ref().unwrap(),
            self.title.as_ref().unwrap()
        )
    }
}

pub fn build_list(milestone: String, issue_labels: Vec<String>) -> Result<IssueList> {
    let client = reqwest::Client::new();
    let zoho_authtoken = env::var("ZOHO_AUTHTOKEN").ok();
    let url =
        format!(
            "https://projectsapi.zoho.com/restapi/portal/{}/projects/{}/bugs/?authtoken={}",
            "marketdojo",
            "328792000000016009",
            zoho_authtoken.unwrap(),
        );
    let mut req = client.get(&url);
    let mut response = req.send()?;
    if !response.status().is_success() {
        bail!("Server error: {:?}", response.status());
    };

    let mut il = response.json::<IssueList>()?;
    il.milestone = Some(milestone);
    il.issue_labels = Some(issue_labels);
    return Ok(il);
}

impl Issue {
    pub fn has_client(&self) -> bool {
        if self.customfields.is_none() {
            return false;
        }
        let cfs = self.customfields.as_ref().unwrap();
        let vec_cfs: Vec<&CustomField> = cfs.iter()
            .filter(|cf| cf.label_name == String::from("From a client:"))
            .collect();
        if vec_cfs.len() > 0 {
            return true;
        }
        false
    }
}
