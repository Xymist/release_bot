pub mod issue;
pub mod task;

use self::{
    issue::{Issue, IssueList},
    task::{Task, TaskList},
};
use errors::*;
use std::{collections::HashMap, rc::Rc};
use zohohorrorshow::client::ZohoClient;
use Config;

#[derive(Debug)]
pub enum Action {
    ZIssue(Issue),
    ZTask(Task),
}

impl Action {
    pub fn name(&self) -> String {
        match self {
            Action::ZIssue(issue) => issue.0.title.clone(),
            Action::ZTask(task) => task.0.name.clone(),
        }
    }

    pub fn display_csv(&self) -> String {
        match self {
            Action::ZIssue(issue) => format!(
                "[{}] {},{},{}",
                issue.0.key,
                issue.0.title,
                issue.0.classification.classification_type,
                issue.0.reported_person
            ),
            Action::ZTask(task) => format!("{},DevelopmentTask,{}", task.0.name, task.0.created_by),
        }
    }

    pub fn display_md(&self) -> String {
        match self {
            Action::ZIssue(issue) => format!(
                "| [{}] {} | {} | {} |",
                issue.0.key,
                issue.0.title,
                issue.0.classification.classification_type,
                issue.0.reported_person
            ),
            Action::ZTask(task) => format!(
                "| {} | DevelopmentTask | {} |",
                task.0.name, task.0.created_by
            ),
        }
    }
}

type ClassifiedActions = HashMap<String, Vec<Action>>;

pub fn zh_client(project_id: i64, config: &Config) -> Result<Rc<ZohoClient>> {
    let mut client = ZohoClient::new(
        &config.zoho_authtoken,
        Some(&config.zoho_organisation),
        None,
    ).chain_err(|| "Could not initialize; exiting")?;

    if let Some(cl) = Rc::get_mut(&mut client) {
        cl.project(project_id);
    };
    Ok(client)
}

pub fn classify_bugs(issues: IssueList) -> ClassifiedActions {
    let bugs = issues.bugs;
    let buglist: Vec<Issue> = bugs.into_iter().filter(|bug| bug.is_closed()).collect();
    let mut client_list: HashMap<String, Vec<Action>> = HashMap::new();

    for bug in buglist {
        let clients: Vec<String> =
            if let (true, &Some(ref cfs)) = (bug.has_client(), &bug.0.customfields) {
                cfs.into_iter()
                    .filter(|cf| cf.label_name == "From a client:")
                    .map(|cf| cf.value.clone())
                    .collect::<Vec<String>>()
            } else if bug.is_feature() {
                vec![String::from("New Features")]
            } else {
                vec![String::from("All Other Changes")]
            };
        for client in clients {
            let blank = Vec::new();
            let client_bugs = client_list.entry(client).or_insert(blank);
            client_bugs.push(Action::ZIssue(bug.clone()))
        }
    }

    client_list
}

pub fn classify_tasks(task_list: TaskList) -> ClassifiedActions {
    let tasks = task_list.tasks;
    let tasklist: Vec<Task> = tasks.into_iter().filter(|task| task.closed_tag()).collect();
    let mut client_list: HashMap<String, Vec<Action>> = HashMap::new();

    for task in tasklist {
        let clients: Vec<String> =
            if let (true, &Some(ref cfs)) = (task.has_client(), &task.0.custom_fields) {
                cfs.into_iter()
                    .filter(|cf| cf.label_name == "From a client:")
                    .map(|cf| cf.value.clone())
                    .collect::<Vec<String>>()
            } else {
                // Tasks are always features, or parts of features
                vec![String::from("New Features")]
            };
        for client in clients {
            let blank = Vec::new();
            let client_tasks = client_list.entry(client).or_insert(blank);
            client_tasks.push(Action::ZTask(task.clone()))
        }
    }

    client_list
}

pub fn merge_actions(
    issue_list: ClassifiedActions,
    mut task_list: ClassifiedActions,
) -> ClassifiedActions {
    for (client, mut bugs) in issue_list {
        let client_tasks = task_list.entry(client).or_insert_with(Vec::new);
        client_tasks.append(&mut bugs)
    }
    task_list
}

#[test]
fn test_merge_actions() {
    let mut list1 = HashMap::new();
    list1.insert("First".to_owned(), vec!["a".to_owned()]);
    let mut list2 = HashMap::new();
    list2.insert("Second".to_owned(), vec!["a".to_owned()]);
    list2.insert("First".to_owned(), vec!["b".to_owned()]);
    let merged: ClassifiedActions = merge_actions(list1, list2);
    assert_eq!(
        merged.get("First"),
        Some(&vec!["b".to_owned(), "a".to_owned()])
    );
    assert_eq!(merged.get("Second"), Some(&vec!["a".to_owned()]));
}

fn separate_and_sort_actions(
    mut client_list: ClassifiedActions,
) -> (Vec<(String, Vec<Action>)>, Vec<Action>, Vec<Action>) {
    let mut features = match client_list.remove("New Features") {
        Some(fs) => fs,
        None => Vec::new(),
    };
    features.sort_by(|a, b| a.name().cmp(&b.name()));

    let mut others = match client_list.remove("All Other Changes") {
        Some(os) => os,
        None => Vec::new(),
    };
    others.sort_by(|a, b| a.name().cmp(&b.name()));

    let mut sorted_client_bugs: Vec<(String, Vec<Action>)> = client_list.into_iter().collect();
    sorted_client_bugs.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
    return (sorted_client_bugs, features, others);
}

pub fn write_actions_csv(client_list: ClassifiedActions) -> String {
    let mut output: String = "".to_owned();
    let (client_bugs, features, others) = separate_and_sort_actions(client_list);

    for (client, bugs) in client_bugs.iter() {
        for bug in bugs.iter() {
            output.push_str(&format!(
                "\n{},{}",
                bug.display_csv(),
                str::replace(client, ",", ";")
            ));
        }
    }

    for feature in features {
        output.push_str(&format!("\n{}", feature.display_csv()));
    }

    for other in others {
        output.push_str(&format!("\n{}", other.display_csv()));
    }

    return output;
}

pub fn write_actions_md(client_list: ClassifiedActions) -> String {
    let mut output: String = "".to_owned();
    let (client_bugs, features, others) = separate_and_sort_actions(client_list);

    for (client, bugs) in client_bugs.iter() {
        for bug in bugs.iter() {
            output.push_str(&format!("\n{} {} |", bug.display_md(), client))
        }
    }

    for feature in features {
        output.push_str(&format!("\n{} |", feature.display_md()));
    }

    for other in others {
        output.push_str(&format!("\n{} |", other.display_md()));
    }

    return output;
}
