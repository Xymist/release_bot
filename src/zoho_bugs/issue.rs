use errors::*;
use std::{fmt, rc::Rc};
use zohohorrorshow::{client::ZohoClient,
                     models::{bug, milestone}};

const CLOSED_STATUSES: &[&str] = &["Tested on Staging", "Tested on Live", "Closed"];

#[derive(Deserialize, Debug, Clone)]
pub struct IssueList {
    pub bugs: Vec<Issue>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Issue(pub bug::Bug);

impl Issue {
    pub fn is_closed(&self) -> bool {
        self.0.closed_tag()
    }

    pub fn has_client(&self) -> bool {
        self.0.has_client()
    }

    pub fn is_feature(&self) -> bool {
        self.0.is_feature()
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "- [{}] {}", self.0.key, self.0.title)
    }
}

pub fn build_list(client: &Rc<ZohoClient>, milestones: Vec<String>) -> Result<IssueList> {
    let mut ms_records = milestones
        .into_iter()
        .map(|m| {
            milestone::milestones(client)
                .status("notcompleted")
                .display_type("all")
                .fetch()
                .unwrap()
                .into_iter()
                .filter(|ms| m == ms.name)
                .collect::<Vec<milestone::Milestone>>()
                .pop()
        })
        .collect::<Vec<Option<milestone::Milestone>>>();

    ms_records.retain(|om| if let Some(ref _m) = *om { true } else { false });

    let ms_ids: Vec<String> = ms_records
        .into_iter()
        .map(|m| m.unwrap().id.to_string())
        .collect();

    let bugs_path = bug::bugs(client).milestone(
        ms_ids
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .as_slice(),
    );

    let bugs = bugs_path.fetch()?;

    let buglist: Vec<Issue> = bugs.into_iter()
        .filter(|bug| bug.closed_tag())
        .map(Issue)
        .collect();

    Ok(IssueList { bugs: buglist })
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
