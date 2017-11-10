use errors::*;
use zoho_bugs::issue::{Issue, Milestone};

pub struct Winnower {
    milestone: Option<String>,
    issue_labels: Option<Vec<String>>,
}

impl Winnower {
    pub fn from_milestone(milestone: String, issue_labels: Vec<String>) -> Result<Winnower> {
        Ok(Winnower {
            milestone: Some(milestone),
            issue_labels: Some(issue_labels),
        })
    }

    pub fn test(&self, issue: &Issue) -> bool {
        let issue_label = issue.key.as_ref().unwrap();
        let issue_milestone = issue.milestone.clone().unwrap_or_default();
        self.issue_labels.as_ref().unwrap().iter().any(|x| {
            x == issue_label
        }) && self.milestone.as_ref().unwrap() == &issue_milestone.name
    }
}