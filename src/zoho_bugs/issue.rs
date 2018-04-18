use Config;
use errors::*;
use std::{fmt, rc::Rc};
use zohohorrorshow::{client::ZohoClient, models::{bug, milestone}};

const CLOSED_STATUSES: &[&str] = &["Tested on Staging", "Tested on Live", "Closed"];

#[derive(Deserialize, Debug, Clone)]
pub struct IssueList {
    pub milestones: Option<Vec<String>>,
    pub bugs: Vec<Issue>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Issue(pub bug::Bug);

impl Issue {
    pub fn is_closed(&self) -> bool {
        self.0.closed_tag()
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "- [{}] {}", self.0.key, self.0.title)
    }
}

pub fn build_list(project_id: i64, milestones: Vec<String>, config: &Config) -> Result<IssueList> {
    let mut client = ZohoClient::new(
        &config.zoho_authtoken,
        Some(&config.zoho_organisation),
        None,
    ).chain_err(|| "Could not initialize; exiting")?;

    if let Some(cl) = Rc::get_mut(&mut client) {
        cl.project(project_id);
    };
    let mut ms_records = milestones
        .clone()
        .into_iter()
        .map(|m| milestone::milestones(&client).by_name(&m).fetch().unwrap())
        .collect::<Vec<Option<milestone::Milestone>>>();
    ms_records.retain(|om| if let Some(ref _m) = *om { true } else { false });
    let ms_ids: Vec<String> = ms_records
        .into_iter()
        .map(|m| m.unwrap().id.to_string())
        .collect();

    let bugs_path = bug::bugs(&client).milestone(&ms_ids
        .iter()
        .map(|s| &**s)
        .collect::<Vec<&str>>()
        .as_slice());

    let bugs = bugs_path.fetch()?;

    Ok(IssueList {
        milestones: Some(milestones),
        bugs: bugs.into_iter().map(Issue).collect::<Vec<Issue>>(),
    })
}

pub trait MDCustomFilters {
    fn has_client(&self) -> bool;
    fn is_feature(&self) -> bool;
    fn issue_type(&self) -> &str;
    fn closed_tag(&self) -> bool;
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

    fn closed_tag(&self) -> bool {
        CLOSED_STATUSES
            .iter()
            .any(|x| *x == self.status.classification_type)
    }
}
