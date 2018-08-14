pub mod issue;
mod issue_iterator;
pub mod task;
mod task_iterator;

use self::{issue::Issue, task::Task};

use errors::*;
use std::rc::Rc;
use zohohorrorshow::client::ZohoClient;
use Config;

pub const CLOSED_STATUSES: &[&str] = &[
    "Tested on Staging",
    "Tested On Staging",
    "Tested on Live",
    "Closed",
];

pub trait MDCustomFilters {
    fn has_client(&self) -> bool;
    fn is_feature(&self) -> bool;
    fn issue_type(&self) -> &str;
    fn closed_tag(&self) -> bool;
    fn milestone(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum Action {
    ZIssue(Issue),
    ZTask(Task),
}

pub struct CustomField {
    label_name: String,
    value: String,
}

impl Action {
    pub fn name(&self) -> String {
        match self {
            Action::ZIssue(issue) => issue.0.title.clone(),
            Action::ZTask(task) => task.0.name.clone(),
        }
    }

    pub fn is_feature(&self) -> bool {
        match self {
            Action::ZIssue(issue) => issue.is_feature(),
            Action::ZTask(task) => task.is_feature(),
        }
    }

    pub fn has_client(&self) -> bool {
        match self {
            Action::ZIssue(issue) => issue.has_client(),
            Action::ZTask(task) => task.has_client(),
        }
    }

    pub fn custom_fields(&self) -> Option<Vec<CustomField>> {
        match self {
            Action::ZIssue(issue) => {
                if issue.0.customfields.is_some() {
                    Some(
                        issue
                            .0
                            .customfields
                            .clone()
                            .unwrap()
                            .iter()
                            .map(|cf| CustomField {
                                label_name: cf.label_name.clone(),
                                value: cf.value.clone(),
                            }).collect(),
                    )
                } else {
                    None
                }
            }
            Action::ZTask(task) => {
                if task.0.custom_fields.is_some() {
                    Some(
                        task.0
                            .custom_fields
                            .clone()
                            .unwrap()
                            .iter()
                            .map(|cf| CustomField {
                                label_name: cf.label_name.clone(),
                                value: cf.value.clone(),
                            }).collect(),
                    )
                } else {
                    None
                }
            }
        }
    }

    pub fn display_csv(&self) -> String {
        match self {
            Action::ZIssue(issue) => format!(
                "[{}] {},{},{}",
                issue.0.key,
                issue.0.title,
                issue.0.issue_type(),
                issue.0.reported_person
            ),
            Action::ZTask(task) => format!(
                "[{}] {},{},{}",
                task.0.key,
                task.0.name,
                task.0.issue_type(),
                task.0.created_person
            ),
        }
    }

    pub fn display_md(&self) -> String {
        match self {
            Action::ZIssue(issue) => format!(
                "| [{}] {} | {} | {} |",
                issue.0.key,
                issue.0.title,
                issue.0.issue_type(),
                issue.0.reported_person
            ),
            Action::ZTask(task) => format!(
                "| [{}] {} | {} | {} |",
                task.0.key,
                task.0.name,
                task.0.issue_type(),
                task.0.created_person
            ),
        }
    }
}

pub struct ClientBug {
    clients: Vec<String>,
    bug: Action,
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
        self.client_bugs
            .sort_by(|a, b| a.clients.len().cmp(&b.clients.len()));
        self.features.sort_by(|a, b| a.name().cmp(&b.name()));
        self.others.sort_by(|a, b| a.name().cmp(&b.name()));

        self
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

pub fn classify_actions(issues: Vec<Action>) -> ClassifiedActions {
    let mut client_list: ClassifiedActions = ClassifiedActions::new();

    for issue in issues {
        if let (true, Some(cfs)) = (issue.has_client(), issue.custom_fields()) {
            let clients: Vec<String> = cfs
                .into_iter()
                .filter(|cf| cf.label_name == "From a client:")
                .nth(0)
                .expect("Somehow a task with clients and custom fields had no client custom field")
                .value
                .split(',')
                .map(|s| s.to_owned())
                .collect();

            client_list.client_bugs.push(ClientBug {
                clients,
                bug: issue.clone(),
            });
        };

        if issue.is_feature() {
            client_list.features.push(issue.clone())
        };

        if !issue.has_client() && !issue.is_feature() {
            client_list.others.push(issue.clone())
        };
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

    for client_bug in &sorted_tickets.client_bugs {
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

    output
}

pub fn write_actions_md(client_list: ClassifiedActions) -> String {
    let sorted_tickets = client_list.sort();

    let mut output: String = "### Client Bugs and Features\n".to_owned();

    output.push_str(
        "\n| Ticket Name | Ticket Type | Raised By | Clients |\n| --- | --- | --- | --- |",
    );

    for client_bug in &sorted_tickets.client_bugs {
        output.push_str(&format!(
            "\n{} {} |",
            client_bug.bug.display_md(),
            client_bug.clients.join(";")
        ))
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

    output
}
