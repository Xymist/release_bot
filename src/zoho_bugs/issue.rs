use Config;
use errors::*;
use std::{fmt, rc::Rc};
use zohohorrorshow::{client::ZohoClient, models::bug};

#[derive(Deserialize, Debug, Clone)]
pub struct IssueList {
    pub milestones: Option<Vec<String>>,
    pub issue_labels: Option<Vec<String>>,
    pub bugs: Vec<Issue>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Issue(pub bug::Bug);

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "- [{}] {}", self.0.key, self.0.title)
    }
}

pub fn build_list(
    project_id: i64,
    milestones: Vec<String>,
    issue_labels: Vec<String>,
    config: &Config,
) -> Result<IssueList> {
    let mut client = ZohoClient::new(
        &config.zoho_authtoken,
        Some(&config.zoho_organisation),
        None,
    ).chain_err(|| "Could not initialize; exiting")?;

    if let Some(cl) = Rc::get_mut(&mut client) {
        cl.project(project_id);
    };

    let bugs = bug::bugs(&client)
        .milestone(&milestones
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .as_slice())
        .fetch()?;

    Ok(IssueList {
        milestones: Some(milestones),
        issue_labels: Some(issue_labels),
        bugs: bugs.into_iter().map(|b| Issue(b)).collect::<Vec<Issue>>(),
    })
}

pub trait MDCustomFilters {
    fn has_client(&self) -> bool;
    fn is_feature(&self) -> bool;
    fn issue_type(&self) -> &str;
}

impl MDCustomFilters for bug::Bug {
    fn has_client(&self) -> bool {
        if self.customfields.is_none() {
            return false;
        }
        let cfs = self.customfields.as_ref().unwrap();
        cfs.iter().any(|cf| cf.label_name == "From a client:")
    }

    fn is_feature(&self) -> bool {
        self.classification.classification_type == "Feature(New)"
    }

    fn issue_type(&self) -> &str {
        &self.classification.classification_type
    }
}
