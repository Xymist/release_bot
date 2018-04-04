use errors::*;
use zoho_bugs::issue::Issue;

pub struct Winnower {
    issue_labels: Option<Vec<String>>,
}

impl Winnower {
    pub fn from_milestone(issue_labels: Vec<String>) -> Result<Winnower> {
        Ok(Winnower {
            issue_labels: Some(issue_labels),
        })
    }

    pub fn test(&self, issue: &Issue) -> bool {
        let closed_statuses: &[&str] = &["Tested on Staging", "Tested on Live", "Closed"];
        let issue_label = &issue.0.key;
        let issue_status = &issue.0.status.classification_type;
        self.issue_labels
            .as_ref()
            .unwrap()
            .iter()
            .any(|x| x == issue_label) || closed_statuses.iter().any(|x| *x == issue_status)
    }
}
