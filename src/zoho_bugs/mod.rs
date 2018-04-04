pub mod issue;
mod winnower;

use self::issue::{Issue, IssueList, MDCustomFilters};
use self::winnower::Winnower;
use errors::*;
use inflector::Inflector;
use std::collections::HashMap;
use zohohorrorshow::models::bug::Customfield;

pub fn print_bugs(issues: IssueList) -> Result<()> {
    let win = Winnower::from_milestone(issues.issue_labels.unwrap().clone())?;
    let bugs = issues.bugs;
    let buglist: Vec<Issue> = bugs.into_iter().filter(|bug| win.test(bug)).collect();
    let mut client_list: HashMap<String, Vec<Issue>> = HashMap::new();
    for bug in buglist {
        // If the Issue is associated with a client, add it to this list, regardless
        // of whether it is a feature. Client bugs have duplicates so that sales
        // can just look for their client's name.
        let clients: Vec<String> = if bug.0.has_client() {
            // .unwrap() is safe here; has_client() has already verified that
            // this is not None.
            let cfs: Vec<Customfield> = bug.0.customfields.clone().unwrap();
            let mut vec_cfs: Vec<String> = cfs.into_iter()
                .filter(|cf| cf.label_name == "From a client:")
                .map(|cf| cf.value)
                .collect();
            let mut cls: Vec<String> = Vec::new();
            vec_cfs
                .into_iter()
                .map(|vc| cls.extend(vc.split(',').map(String::from).collect(): Vec<String>))
                .count();
            cls.into_iter().map(|st| st.to_title_case()).collect()
        // Non-client Issues may be new features we're adding to the app; these
        // are of interest to everyone. Features requested by clients will be in
        // the previous set, and not duplicated here.
        } else if bug.0.is_feature() {
            vec![String::from("New Features")]
        // All other work done is listed last. These Issues are not associated
        // with a client, nor are they new features, but they are still valuable.
        // This includes changes such as minor fixes, proactive debugging,
        // infrastructure changes, experiments and efficiency improvements.
        } else {
            vec![String::from("All Other Changes")]
        };
        for client in clients {
            let blank = Vec::new();
            let client_bugs = client_list.entry(client).or_insert(blank);
            client_bugs.push(bug.clone())
        }
    }

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
