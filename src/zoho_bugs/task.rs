use crate::errors::*;
use crate::zoho_bugs::{Action, MDCustomFilters, CLOSED_STATUSES};
use serde_derive::Deserialize;
use zohohorrorshow::prelude::*;

#[derive(Deserialize, Debug, Clone)]
pub struct Task(pub zoho_task::Task);

impl Task {
    pub fn has_client(&self) -> bool {
        self.0.has_client()
    }

    pub fn is_feature(&self) -> bool {
        self.0.is_feature()
    }
}

pub fn build_list(client: &ZohoClient, milestones: &[String]) -> Result<Vec<Action>> {
    use zoho_tasklist::{Filter, Flag};

    let tl_ids: Vec<i64> = client
        .tasklists()
        .filter(Filter::Flag(Flag::Internal))
        .iter_get()
        .filter(std::result::Result::is_ok)
        .map(std::result::Result::unwrap)
        .filter_map(|t| {
            if milestones.contains(&t.milestone.name.trim().to_owned()) {
                return Some(t.id);
            }
            None
        })
        .collect();

    let closed_tasks: Vec<Action> = client
        .tasks()
        .iter_get()
        .filter(std::result::Result::is_ok)
        .map(std::result::Result::unwrap)
        .filter(MDCustomFilters::closed_tag)
        .filter(|t| {
            let ms = &t.milestone().trim().to_owned();
            milestones.contains(ms)
                || tl_ids.contains(&t.tasklist_id)
                || tl_ids.contains(&t.clone().tasklist.unwrap_or_default().id)
        })
        .map(Task)
        .map(Action::ZTask)
        .collect();

    Ok(closed_tasks)
}

impl MDCustomFilters for zoho_task::Task {
    fn closed_tag(&self) -> bool {
        CLOSED_STATUSES
            .iter()
            .any(|x| *x == self.status.name.to_lowercase().trim())
    }

    fn has_client(&self) -> bool {
        if self.custom_fields.is_none() {
            return false;
        }
        let cfs = self.custom_fields.as_ref().unwrap();
        cfs.iter()
            .any(|cf| cf.label_name.to_lowercase().contains("from a client"))
    }

    fn milestone(&self) -> String {
        if self.custom_fields.is_some() {
            let cfs = self.custom_fields.as_ref().unwrap();
            if cfs.iter().any(|cf| cf.label_name == "Release Milestone") {
                return cfs
                    .iter()
                    .find(|cf| cf.label_name == "Release Milestone")
                    .expect("Failed to extract Release Milestone")
                    .value
                    .clone();
            }
        }
        "".to_owned()
    }

    fn issue_type(&self) -> &str {
        "DevelopmentTask"
    }

    fn is_feature(&self) -> bool {
        true
    }
}
