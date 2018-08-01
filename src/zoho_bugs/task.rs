use errors::*;
use std::rc::Rc;
use zohohorrorshow::{
    client::ZohoClient,
    models::{milestone, task, tasklist},
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
    let ms_ids: Vec<i64> = milestone::milestones(client)
        .status("notcompleted")
        .display_type("all")
        .fetch()
        .expect("Failed to retrieve list of milestones")
        .into_iter()
        .filter(|ms| milestones.contains(&ms.name))
        .map(|m| m.id)
        .collect();

    let mut tasklists = tasklist::tasklists(client).flag("internal").fetch()?;
    let task_ids: Vec<i64> = tasklists
        .iter()
        .flat_map(|t| {
            tasklist::tasklists(client)
                .by_id(t.id)
                .tasks()
                .fetch()
                .expect(&format!("Failed to fetch tasks for tasklist {}:", t.name))
                .into_iter()
                .filter(|t| t.closed_tag())
                .map(|t| t.id)
        }).collect();

    let tasks: Vec<task::Task> = task_ids
        .iter()
        .map(|tid| {
            task::tasks(client)
                .by_id(*tid)
                .fetch()
                .expect(&format!("Failed to fetch task {}", tid))
                .remove(0)
        }).collect();

    tasklists.retain(|t| ms_ids.contains(&t.milestone.id));
    let tl_ids: Vec<i64> = tasklists.into_iter().map(|m| m.id).collect();
    let closed_tasks: Vec<Task> = tasks
        .into_iter()
        .filter(|t| {
            milestones.contains(&t.milestone())
                || tl_ids.contains(&t.tasklist_id)
                || tl_ids.contains(&t.clone().tasklist.unwrap_or_default().id)
        }).map(Task)
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
                    .map(|cf| cf.value.clone())
                    .collect::<Vec<String>>()
                    .join("");
            }
        }
        return "".to_owned();
    }
}
