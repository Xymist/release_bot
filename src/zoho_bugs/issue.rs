use crate::errors::*;
use std::rc::Rc;
use crate::zoho_bugs::issue_iterator::IssueIterator;
use crate::zoho_bugs::{Action, MDCustomFilters, CLOSED_STATUSES};
use zohohorrorshow::{
    client::ZohoClient,
    models::{bug, milestone},
};

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

pub fn build_list(client: &Rc<ZohoClient>, milestones: &[String]) -> Result<Vec<Action>> {
    let ms_ids: Vec<String> = milestones
        .iter()
        .filter_map(|m| {
            let p_m = milestone::milestones(client)
                .status("notcompleted")
                .display_type("all")
                .fetch()
                .expect("Failed to retrieve milestone list")
                .into_iter()
                .find(|ms| m == ms.name.trim());

            match p_m {
                Some(n) => Some(n.id.to_string()),
                None => None
            }
        })
        .collect();

    let buglist: Vec<Action> = IssueIterator::new(&client.clone(), ms_ids)
        .filter_map(Result::ok)
        .peekable()
        .filter(|bug| bug.is_closed())
        .map(Action::ZIssue)
        .collect();

    Ok(buglist)
}

impl MDCustomFilters for bug::Bug {
    fn has_client(&self) -> bool {
        if self.customfields.is_none() {
            return false;
        }
        let cfs = self.customfields.as_ref().unwrap();
        cfs.iter()
            .any(|cf| cf.label_name.to_lowercase().contains("from a client"))
    }

    fn is_feature(&self) -> bool {
        self.classification.classification_type == "Feature(New)"
            || self.classification.classification_type == "Enhancement"
    }

    fn issue_type(&self) -> &str {
        &self.classification.classification_type
    }

    fn closed_tag(&self) -> bool {
        CLOSED_STATUSES
            .iter()
            .any(|x| *x == self.status.classification_type.to_lowercase().trim())
    }

    // Bugs from a milestone (as we use them here) do not have a milestone
    // attached when they are provided from the API.
    fn milestone(&self) -> String {
        unimplemented!()
    }
}
