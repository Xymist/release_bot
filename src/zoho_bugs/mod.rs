pub mod issue;
mod winnower;

use std::collections::HashMap;
use self::issue::{Issue, IssueList, CustomField};
use self::winnower::Winnower;
use errors::*;
use inflector::Inflector;

pub fn print_bugs(issues: IssueList) -> Result<()> {
    let win = Winnower::from_milestone(
        issues.milestone.unwrap().clone(),
        issues.issue_labels.unwrap().clone(),
    )?;
    let bugs = issues.bugs.unwrap();
    let buglist: Vec<Issue> = bugs.into_iter().filter(|bug| win.test(bug)).collect();
    println!("\n### This milestone contained {} issues:\n", buglist.len());
    let mut clients: HashMap<String, Vec<Issue>> = HashMap::new();
    for bug in buglist {
        let client: String = if bug.has_client() {
            let cfs = bug.customfields.clone().unwrap();
            let mut vec_cfs: Vec<CustomField> = cfs.into_iter()
                .filter(|cf| cf.label_name == String::from("From a client:"))
                .collect();
            vec_cfs.remove(0).value.to_title_case()
        } else {
            String::from("No Associated Customer")
        };

        let client_bugs = clients.entry(client).or_insert(vec![]);
        client_bugs.push(bug)

    }

    let mut sortable: Vec<(&String, &Vec<Issue>)> = clients.iter().collect();
    sortable.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
    for (client, client_bugs) in sortable {
        println!("\n#### {} ({})", client, client_bugs.len());
        for client_bug in client_bugs.into_iter() {
            println!("{}", client_bug);
        }
    }
    Ok(())

}