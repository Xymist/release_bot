pub mod issue;
pub mod task;

use self::issue::{Issue, IssueList, MDCustomFilters};
use errors::*;
use std::collections::HashMap;

type ClassBugs = HashMap<String, Vec<Issue>>;

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
