pub mod issue;
pub mod task;

use self::{issue::{Issue, IssueList, MDCustomFilters},task::{Task, TaskList, MDCustomFilters}};
use Config;
use errors::*;
use std::{collections::HashMap, rc::Rc};
use zohohorrorshow::client::ZohoClient;

type ClassBugs = HashMap<String, Vec<Issue>>;
type ClassTasks = HashMap<String, Vec<Task>>;

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

pub fn classify_bugs(issues: IssueList) -> ClassBugs {
    let bugs = issues.bugs;
    let buglist: Vec<Issue> = bugs.into_iter().filter(|bug| bug.is_closed()).collect();
    let mut client_list: HashMap<String, Vec<Issue>> = HashMap::new();

    for bug in buglist {
        let clients: Vec<String> =
            if let (true, &Some(ref cfs)) = (bug.0.has_client(), &bug.0.customfields) {
                cfs.into_iter()
                    .filter(|cf| cf.label_name == "From a client:")
                    .map(|cf| &cf.value)
                    .map(|vc| vc.split(',').map(String::from).collect())
                    .collect::<Vec<String>>()
            } else if bug.0.is_feature() {
                vec![String::from("New Features")]
            } else {
                vec![String::from("All Other Changes")]
            };
        for client in clients {
            let blank = Vec::new();
            let client_bugs = client_list.entry(client).or_insert(blank);
            client_bugs.push(bug.clone())
        }
    }

    client_list
}

pub fn classify_tasks(task_list: TaskList) -> ClassTasks {
    let tasks = task_list.tasks;
    let mut client_list: HashMap<String, Vec<Issue>> = HashMap::new();

    for task in tasklist {
        let clients: Vec<String> =
            if let (true, &Some(ref cfs)) = (task.0.has_client(), &task.0.customfields) {
                cfs.into_iter()
                    .filter(|cf| cf.label_name == "From a client:")
                    .map(|cf| &cf.value)
                    .map(|vc| vc.split(',').map(String::from).collect())
                    .collect::<Vec<String>>()
            } else if task.0.is_feature() {
                vec![String::from("New Features")]
            } else {
                vec![String::from("All Other Changes")]
            };
        for client in clients {
            let blank = Vec::new();
            let client_tasks = client_list.entry(client).or_insert(blank);
            client_tasks.push(task.clone())
        }
    }

    client_list
}

pub fn insert_tasks(mut task_list: ClassTasks) -> Result<()> {
    unimplemented!()
}

pub fn print_bugs(mut client_list: ClassBugs) -> Result<()> {
    let features = match client_list.remove("New Features") {
        Some(fs) => fs,
        None => Vec::new(),
    };

    let others = match client_list.remove("All Other Changes") {
        Some(os) => os,
        None => Vec::new(),
    };

    let mut sortable: Vec<(&String, &Vec<Issue>)> = client_list.iter().collect();
    sortable.sort_by(|a, b| a.1.len().cmp(&b.1.len()));

    for (client, client_bugs) in sortable {
        println!("\n#### {} ({})\n", client, client_bugs.len());
        for client_bug in client_bugs.iter() {
            println!("{}", client_bug);
        }
    }

    if !features.is_empty() {
        println!("\n### New Features ({})\n", features.len());
        for feature in &features {
            println!("{}", feature);
        }
    }

    println!("\n### All Other Changes ({})\n", others.len());
    for other in &others {
        println!("{}", other);
    }

    Ok(())
}
