use errors::*;
use zoho_bugs::issue::Issue;

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
        let closed_statuses: &[&str] = &["Tested on Staging", "Tested on Live", "Closed"];
        let issue_label = issue.key.as_ref().unwrap();
        let issue_milestone = issue.milestone.clone().unwrap_or_default();
        let issue_status = issue.status.clone().unwrap().type_name.unwrap();
        // Either we addressed an issue, but it's in the wrong milestone...
        self.issue_labels.as_ref().unwrap().iter().any(|x| {
            x == issue_label
        }) || // Or we closed an issue in this milestone but may not have tagged it.
            (closed_statuses.iter().any(|x| *x == issue_status) &&
                 self.milestone.as_ref().unwrap() == &issue_milestone.name)
    }
}