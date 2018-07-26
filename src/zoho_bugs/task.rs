use errors::*;
use std::{fmt, rc::Rc};
use zohohorrorshow::{client::ZohoClient,
                     models::{milestone, task, tasklist}};

const CLOSED_STATUSES: &[&str] = &["Tested on Staging", "Tested on Live", "Closed"];

#[derive(Deserialize, Debug, Clone)]
pub struct TaskList {
    pub tasks: Vec<Task>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Task(pub task::Task);

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "- {}", self.0.name)
    }
}

impl Task {
    pub fn closed_tag(&self) -> bool {
        self.0.closed_tag()
    }

    pub fn has_client(&self) -> bool {
        self.0.has_client()
    }
}

pub fn build_list(client: &Rc<ZohoClient>, milestones: Vec<String>) -> Result<TaskList> {
    let mut ms_records = milestones
        .into_iter()
        .map(|m| {
            milestone::milestones(client)
                .status("notcompleted")
                .display_type("all")
                .fetch()
                .expect("Failed to retrieve list of tasks")
                .into_iter()
                .filter(|ms| m == ms.name)
                .collect::<Vec<milestone::Milestone>>()
                .pop()
        })
        .collect::<Vec<Option<milestone::Milestone>>>();

    ms_records.retain(|om| if let Some(ref _m) = *om { true } else { false });

    let ms_ids: Vec<i64> = ms_records.into_iter().map(|m| m.unwrap().id).collect();

    let mut tasklists = tasklist::tasklists(client).flag("internal").fetch()?;
    tasklists.retain(|t| ms_ids.contains(&t.milestone.id));
    let tasks: Vec<task::Task> = tasklists
        .into_iter()
        .flat_map(|t| {
            tasklist::tasklists(client)
                .by_id(t.id)
                .tasks()
                .fetch()
                .expect(&format!("Failed to fetch tasks for tasklist {}:", t.name))
        })
        .collect();
    let closed_tasks: Vec<Task> = tasks
        .into_iter()
        .filter(|t| t.closed_tag())
        .map(Task)
        .collect();

    Ok(TaskList {
        tasks: closed_tasks,
    })
}

pub trait MDCustomFilters {
    fn closed_tag(&self) -> bool;
    fn has_client(&self) -> bool;
}

impl MDCustomFilters for task::Task {
    fn closed_tag(&self) -> bool {
        CLOSED_STATUSES
            .iter()
            .any(|x| *x == self.status.status_type)
    }

    fn has_client(&self) -> bool {
        if self.custom_fields.is_none() {
            return false;
        }
        let cfs = self.custom_fields.as_ref().unwrap();
        cfs.iter().any(|cf| cf.label_name == "From a client:")
    }
}
