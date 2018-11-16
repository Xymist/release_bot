use crate::errors::*;
use std::rc::Rc;
use crate::zoho_bugs::task_iterator::TaskIterator;
use crate::zoho_bugs::{Action, MDCustomFilters, CLOSED_STATUSES};
use zohohorrorshow::{
    client::ZohoClient,
    models::{task, tasklist},
};

#[derive(Deserialize, Debug, Clone)]
pub struct Task(pub task::Task);

impl Task {
    pub fn closed_tag(&self) -> bool {
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
    let tl_ids: Vec<i64> = tasklist::tasklists(client)
        .flag("internal")
        .fetch()
        .expect("Failed to fetch Tasklists")
        .into_iter()
        .filter_map(|t| {
            if milestones.contains(&t.milestone.name.trim().to_owned()) {
                return Some(t.id)
            }
            None
        })
        .collect();

    let closed_tasks: Vec<Action> = TaskIterator::new(&client.clone())
        .filter_map(Result::ok)
        .peekable()
        .filter(|t| t.closed_tag())
        .filter(|t| {
            milestones.contains(&t.0.milestone().trim().to_owned())
                || tl_ids.contains(&t.0.tasklist_id)
                || tl_ids.contains(&t.0.clone().tasklist.unwrap_or_default().id)
        })
        .map(Action::ZTask)
        .collect();

    Ok(closed_tasks)
}

impl MDCustomFilters for task::Task {
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
