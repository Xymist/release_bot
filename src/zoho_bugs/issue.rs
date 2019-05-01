use crate::errors::*;
use crate::zoho_bugs::{Action, MDCustomFilters, CLOSED_STATUSES};
use serde_derive::Deserialize;
use zohohorrorshow::prelude::*;

#[derive(Deserialize, Debug, Clone)]
pub struct Issue(pub zoho_bug::Bug);

impl Issue {
    pub fn has_client(&self) -> bool {
        self.0.has_client()
    }

    pub fn is_feature(&self) -> bool {
        self.0.is_feature()
    }
}

pub fn build_list(client: &ZohoClient, milestone_names: &[String]) -> Result<Vec<Action>> {
    use zoho_bug;
    use zoho_milestone::{DisplayType, Status};

    let maybe_milestones = client
        .milestones()
        .filter(zoho_milestone::Filter::Status(Status::NotCompleted))
        .filter(zoho_milestone::Filter::DisplayType(DisplayType::All))
        .get()
        .unwrap_or(None);

    if maybe_milestones.is_none() {
        return Ok(Vec::new());
    }

    let milestones = maybe_milestones.unwrap().milestones;

    let ms_ids: Vec<i64> = milestone_names
        .iter()
        .filter_map(|name| {
            let found = milestones.iter().find(|ms| *name == ms.name.trim());

            match found {
                Some(ms) => Some(ms.id),
                None => None,
            }
        })
        .collect();

    if ms_ids.is_empty() {
        return Ok(Vec::new());
    }

    let request = client
        .bugs()
        .filter(zoho_bug::Filter::Milestone(ms_ids))
        .filter(zoho_bug::Filter::SortColumn(
            zoho_bug::SortColumn::LastModifiedTime,
        ))
        .filter(zoho_bug::Filter::SortOrder(zoho_bug::SortOrder::Descending));

    let buglist = request
        .iter_get()
        .filter(std::result::Result::is_ok)
        .map(std::result::Result::unwrap)
        .filter(MDCustomFilters::closed_tag)
        .map(Issue)
        .map(Action::ZIssue)
        .collect();

    Ok(buglist)
}

impl MDCustomFilters for zoho_bug::Bug {
    fn has_client(&self) -> bool {
        if let Some(ref cfs) = self.customfields {
            return cfs
                .iter()
                .any(|cf| cf.label_name.to_lowercase().contains("from a client"));
        }
        false
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
