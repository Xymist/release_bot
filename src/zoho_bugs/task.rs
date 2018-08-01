use errors::*;
use std::rc::Rc;
use zoho_bugs::task_iterator::TaskIterator;
use zohohorrorshow::{
    client::ZohoClient,
    models::{task, tasklist},
};

const CLOSED_STATUSES: &[&str] = &["Tested On Staging", "Tested on Live", "Closed"];

#[derive(Deserialize, Debug, Clone)]
pub struct TaskList {
    pub tasks: Vec<Task>,
}

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
        true
    }
}

pub fn build_list(client: &Rc<ZohoClient>, milestones: Vec<String>) -> Result<TaskList> {
    let tl_ids: Vec<i64> = tasklist::tasklists(client)
        .flag("internal")
        .fetch()
        .expect("Failed to fetch Tasklists")
        .into_iter()
        .filter(|t| milestones.contains(&t.milestone.name))
        .map(|tl| tl.id)
        .collect();

    let closed_tasks: Vec<Task> = TaskIterator::new(&client.clone()).filter_map(Result::ok)
        .peekable()
        .filter(|t| t.closed_tag())
        .filter(|t| {
            milestones.contains(&t.0.milestone())
                || tl_ids.contains(&t.0.tasklist_id)
                || tl_ids.contains(&t.0.clone().tasklist.unwrap_or_default().id)
        })
        .collect();

    Ok(TaskList {
        tasks: closed_tasks,
    })
}

pub trait MDCustomFilters {
    fn closed_tag(&self) -> bool;
    fn has_client(&self) -> bool;
    fn milestone(&self) -> String;
}

impl MDCustomFilters for task::Task {
    fn closed_tag(&self) -> bool {
        CLOSED_STATUSES.iter().any(|x| *x == self.status.name)
    }

    fn has_client(&self) -> bool {
        if self.custom_fields.is_none() {
            return false;
        }
        let cfs = self.custom_fields.as_ref().unwrap();
        cfs.iter().any(|cf| cf.label_name == "From a Client")
    }

    fn milestone(&self) -> String {
        if self.custom_fields.is_some() {
            let cfs = self.custom_fields.as_ref().unwrap();
            if cfs.iter().any(|cf| cf.label_name == "Release Milestone") {
                return cfs
                    .iter()
                    .filter(|cf| cf.label_name == "Release Milestone")
                    .nth(0)
                    .expect("Failed to extract Release Milestone")
                    .value
                    .clone();
            }
        }
        return "".to_owned();
    }
}
