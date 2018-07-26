pub mod issue;
pub mod task;

use self::{issue::{Issue, IssueList},
           task::{Task, TaskList}};
use errors::*;
use std::{collections::HashMap, rc::Rc};
use zohohorrorshow::client::ZohoClient;
use Config;

type ClassifiedActions = HashMap<String, Vec<String>>;

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
    let mut client_list: HashMap<String, Vec<String>> = HashMap::new();

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
            client_bugs.push(format!("{}", bug))
        }
    }

    client_list
}

pub fn classify_tasks(task_list: TaskList) -> ClassifiedActions {
    let tasks = task_list.tasks;
    let tasklist: Vec<Task> = tasks.into_iter().filter(|task| task.closed_tag()).collect();
    let mut client_list: HashMap<String, Vec<String>> = HashMap::new();

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
            client_tasks.push(format!("{}", task))
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

pub fn print_actions(mut client_list: ClassifiedActions) -> Result<()> {
    let mut features = match client_list.remove("New Features") {
        Some(fs) => fs,
        None => Vec::new(),
    };

    let mut others = match client_list.remove("All Other Changes") {
        Some(os) => os,
        None => Vec::new(),
    };

    let mut sorted_client_bugs: Vec<(&String, &Vec<String>)> = client_list.iter().collect();
    sorted_client_bugs.sort_by(|a, b| a.1.len().cmp(&b.1.len()));

    for (client, client_bugs) in sorted_client_bugs {
        for client_bug in client_bugs.iter() {
            println!("{} {} |", client_bug, client);
        }
    }

    if !features.is_empty() {
        features.sort();
        for feature in &features {
            println!("{} |", feature);
        }
    }

    others.sort();
    for other in &others {
        println!("{} |", other);
    }

    Ok(())
}
