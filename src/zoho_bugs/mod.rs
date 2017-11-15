pub mod issue;
mod winnower;

use std::collections::HashMap;
use self::issue::{Issue, IssueList, CustomField};
use self::winnower::Winnower;
use errors::*;

pub fn print_bugs(issues: IssueList) -> Result<()> {
    let win = Winnower::from_milestone(
        issues.milestone.unwrap().clone(),
        issues.issue_labels.unwrap().clone(),
    )?;
    let bugs = issues.bugs.unwrap();
    let buglist: Vec<Issue> = bugs.into_iter().filter(|bug| win.test(bug)).collect();
    let mut client_list: HashMap<String, Vec<Issue>> = HashMap::new();
    for bug in buglist {
        let clients: Vec<String> = if bug.has_client() {
            let cfs: Vec<CustomField> = bug.customfields.clone().unwrap();
            let mut vec_cfs: Vec<String> = cfs.into_iter()
                .filter(|cf| cf.label_name == "From a client:")
                .map(|cf| cf.value)
                .collect();
            vec_cfs
        } else {
            vec![String::from("No Associated Customer")]
        };
        for client in clients {
            let blank = Vec::new();
            let client_bugs = client_list.entry(client).or_insert(blank);
            client_bugs.push(bug.clone())
        }
    }

    let mut sortable: Vec<(&String, &Vec<Issue>)> = client_list.iter().collect();
    sortable.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
    for (client, client_bugs) in sortable {
        println!("\n#### {} ({})\n", client, client_bugs.len());
        for client_bug in client_bugs.iter() {
            println!("{}", client_bug);
        }
    }
    Ok(())

}