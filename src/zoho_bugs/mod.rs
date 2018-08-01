pub mod issue;
pub mod task;

use self::{
    issue::{Issue, IssueList},
    task::{Task, TaskList},
};
use errors::*;
use std::rc::Rc;
use zohohorrorshow::client::ZohoClient;
use Config;

#[derive(Debug, Clone)]
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

    // pub fn is_feature(&self) -> bool {
    //     match self {
    //         Action::ZIssue(issue) => issue.is_feature(),
    //         Action::ZTask(_) => true,
    //     }
    // }

    pub fn has_client(&self) -> bool {
        match self {
            Action::ZIssue(issue) => issue.has_client(),
            Action::ZTask(task) => task.has_client(),
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
            Action::ZTask(task) => format!(
                "[{}] {},DevelopmentTask,{}",
                task.0.key, task.0.name, task.0.created_person
            ),
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
                "| [{}] {} | DevelopmentTask | {} |",
                task.0.key, task.0.name, task.0.created_person
            ),
        }
    }
}

pub struct ClientBug {
    clients: Vec<String>,
    bug: Action
}

pub struct ClassifiedActions {
    client_bugs: Vec<ClientBug>,
    features: Vec<Action>,
    others: Vec<Action>,
}

impl ClassifiedActions {
    pub fn new() -> ClassifiedActions {
        ClassifiedActions {
            client_bugs: vec![],
            features: vec![],
            others: vec![],
        }
    }

    pub fn sort(mut self) -> Self {
        self.client_bugs.sort_by(|a, b| a.clients.len().cmp(&b.clients.len()));
        self.features.sort_by(|a, b| a.name().cmp(&b.name()));
        self.others.sort_by(|a, b| a.name().cmp(&b.name()));

        return self;
    }
}

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
    let mut client_list: ClassifiedActions = ClassifiedActions::new();

    for bug in buglist {
        if let (true, &Some(ref cfs)) = (bug.has_client(), &bug.0.customfields) {
            let clients: Vec<String> = cfs.into_iter()
                .filter(|cf| cf.label_name == "From a client:")
                .nth(0)
                .expect("Somehow a task with clients and custom fields had no client custom field")
                .value
                .split(",")
                .map(|s| s.to_owned())
                .collect();

            client_list.client_bugs.push(ClientBug {
                clients: clients,
                bug: Action::ZIssue(bug.clone())
            });
        };

        if bug.is_feature() {
            client_list.features.push(Action::ZIssue(bug.clone()))
        };

        if !bug.has_client() && !bug.is_feature() {
            client_list.others.push(Action::ZIssue(bug.clone()))
        };
    }

    client_list
}

pub fn classify_tasks(task_list: TaskList) -> ClassifiedActions {
    let tasks = task_list.tasks;
    let tasklist: Vec<Task> = tasks.into_iter().filter(|task| task.closed_tag()).collect();
    let mut client_list: ClassifiedActions = ClassifiedActions::new();

    for task in tasklist {
        if let (true, &Some(ref cfs)) = (task.has_client(), &task.0.custom_fields) {
            let clients: Vec<String> = cfs.into_iter()
                .filter(|cf| cf.label_name == "From a client:")
                .nth(0)
                .expect("Somehow a task with clients and custom fields had no client custom field")
                .value
                .split(",")
                .map(|s| s.to_owned())
                .collect();

            client_list.client_bugs.push(ClientBug {
                clients: clients,
                bug: Action::ZTask(task.clone())
            });
        }

        client_list.features.push(Action::ZTask(task.clone()));
    }

    client_list
}

pub fn merge_actions(
    mut issue_list: ClassifiedActions,
    mut task_list: ClassifiedActions,
) -> ClassifiedActions {
    issue_list.client_bugs.append(&mut task_list.client_bugs);
    issue_list.features.append(&mut task_list.features);
    issue_list.others.append(&mut task_list.others);
    issue_list
}

pub fn write_actions_csv(client_list: ClassifiedActions) -> String {
    let mut output: String = "".to_owned();
    let sorted_tickets = client_list.sort();

    for client_bug in sorted_tickets.client_bugs.iter() {
        output.push_str(&format!(
            "\n{},{}",
            client_bug.bug.display_csv(),
            client_bug.clients.join(";")
        ));
    }

    for feature in sorted_tickets.features {
        if !feature.has_client() {
            output.push_str(&format!("\n{}", feature.display_csv()));
        }
    }

    for other in sorted_tickets.others {
        output.push_str(&format!("\n{}", other.display_csv()));
    }

    return output;
}

pub fn write_actions_md(client_list: ClassifiedActions) -> String {
    let sorted_tickets = client_list.sort();

    let mut output: String = "### Client Bugs and Features\n".to_owned();

    output.push_str(
        "\n| Ticket Name | Ticket Type | Raised By | Clients |\n| --- | --- | --- | --- |",
    );

    for client_bug in sorted_tickets.client_bugs.iter() {
        output.push_str(&format!("\n{} {} |", client_bug.bug.display_md(), client_bug.clients.join(";")))
    }

    output.push_str(
        "\n\n### Features and Enhancements\n\n| Ticket Name | Ticket Type | Raised By |\n| --- | --- | --- |",
    );

    for feature in sorted_tickets.features {
        output.push_str(&format!("\n{}", feature.display_md()));
    }

    output.push_str(
        "\n\n### Other Bugs\n\n| Ticket Name | Ticket Type | Raised By |\n| --- | --- | --- |",
    );

    for other in sorted_tickets.others {
        output.push_str(&format!("\n{}", other.display_md()));
    }

    return output;
}
