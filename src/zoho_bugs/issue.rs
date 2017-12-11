use errors::*;
use std::fmt;
use reqwest;
use Config;

// FIXME: This only exists because the Zoho API returns an object containing an
// array of bugs, rather than just an array of bugs. Maybe Serde has a way
// around this?
#[derive(Deserialize, Debug)]
pub struct IssueList {
    pub milestone: Option<String>,
    pub issue_labels: Option<Vec<String>>,
    pub bugs: Option<Vec<Issue>>,
}

#[derive(Deserialize, Debug)]
pub struct MilestoneList {
    pub milestones: Option<Vec<Milestone>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Issue {
    pub key: Option<String>,
    pub title: Option<String>,
    pub milestone: Option<IssueMilestone>,
    pub customfields: Option<Vec<CustomField>>,
    pub status: Option<Status>,
    pub classification: Option<Classification>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Classification {
    id: Option<u64>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Status {
    color_code: Option<String>,
    id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
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
    pub id: u64,
}

impl Default for Milestone {
    fn default() -> Milestone {
        Milestone {
            name: String::from("No Milestone"),
            id: 0,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct IssueMilestone {
    pub name: String,
    pub id: String,
}

impl Default for IssueMilestone {
    fn default() -> IssueMilestone {
        IssueMilestone {
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

pub fn build_list(
    project_id: &str,
    milestone: String,
    issue_labels: Vec<String>,
    config: &Config,
) -> Result<IssueList> {
    let client = reqwest::Client::new();
    let zoho_authtoken = &config.zoho_authtoken;
    let milestone_url =
        format!(
            "https://projectsapi.zoho.com/restapi/portal/{}/projects/{}/milestones/?authtoken={}",
            &config.zoho_organisation,
            project_id,
            zoho_authtoken,
        );
    let mut milestone_req = client.get(&milestone_url);
    let mut milestone_response = milestone_req.send()?;
    if !milestone_response.status().is_success() {
        bail!("Server error: {:?}", milestone_response.status());
    };
    let msl = milestone_response.json::<MilestoneList>()?;
    let ms = match msl.milestones {
        Some(ms) => ms.into_iter().find(|m| m.name == milestone),
        None => bail!("No Milestone Found!"),
    };

    let bugs_url =
        format!(
            "https://projectsapi.zoho.com/restapi/portal/{}/projects/{}/bugs/?authtoken={}&milestone=[{}]",
            &config.zoho_organisation,
            project_id,
            zoho_authtoken,
            ms.unwrap().id,
        );
    let mut bugs_req = client.get(&bugs_url);
    let mut bugs_response = bugs_req.send()?;
    if !bugs_response.status().is_success() {
        bail!("Server error: {:?}", bugs_response.status());
    };
    let mut il = bugs_response.json::<IssueList>()?;

    il.milestone = Some(milestone);
    il.issue_labels = Some(issue_labels);
    Ok(il)
}

impl Issue {
    pub fn has_client(&self) -> bool {
        if self.customfields.is_none() {
            return false;
        }
        let cfs = self.customfields.as_ref().unwrap();
        cfs.iter().any(|cf| cf.label_name == "From a client:")
    }

    pub fn is_feature(&self) -> bool {
        if self.classification.is_none() {
            return false;
        }
        self.issue_type() == "Feature(New)"
    }

    pub fn issue_type(&self) -> String {
        let class = self.classification.as_ref().unwrap();
        class.clone().type_name.unwrap()
    }
}
