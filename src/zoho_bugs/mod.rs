pub mod issue;
pub mod task;

use crate::config::{Config, Project};
use color_eyre::{eyre::eyre, Result};
use regex::Regex;
use std::fmt::Write as _;
use std::sync::OnceLock;
use zohohorrorshow::prelude::*;

// Flagging issues and tasks as closed uses custom fields, which are not necessarily consistently
// named. This should be an exhaustive list of those statuses which indicate that QA is happy with
// the ticket as it stands.
pub const CLOSED_STATUSES: &[&str] = &["tested on staging", "tested on live", "closed"];

static QUOTES: OnceLock<Regex> = OnceLock::new();

// Filters which are required for this application but which ZohoHorrorshow does not provide since
// they are dependent on factors defined in MD's Zoho Project settings
pub trait MDCustomFilters {
    fn has_client(&self) -> bool;
    fn is_feature(&self) -> bool;
    fn issue_type(&self) -> &str;
    fn closed_tag(&self) -> bool;
    fn milestone(&self) -> String;
}

// Issues and Tasks are the two possible entities within the Zoho Project on which a developer
// may take action
#[derive(Debug, Clone)]
pub enum Action {
    ZIssue(issue::Issue),
    ZTask(task::Task),
}

pub struct CustomField {
    label_name: String,
    value: String,
}

impl From<&zohohorrorshow::models::bug::Customfield> for CustomField {
    fn from(cf: &zohohorrorshow::models::bug::Customfield) -> CustomField {
        CustomField {
            label_name: cf.label_name.clone(),
            value: cf.value.clone(),
        }
    }
}

impl From<&zohohorrorshow::models::task::CustomField> for CustomField {
    fn from(cf: &zohohorrorshow::models::task::CustomField) -> CustomField {
        CustomField {
            label_name: cf.label_name.clone(),
            value: cf.value.clone(),
        }
    }
}

impl Action {
    // Despite being part of the same system, [`Issue`]s are defined by Zoho as having a title, whilst
    // [`Task`]s are defined as having a name. They serve the same purpose so this is abstracted away.
    pub fn name(&self) -> String {
        match self {
            Action::ZIssue(issue) => issue.0.title.clone(),
            Action::ZTask(task) => task.0.name.clone(),
        }
    }

    // Features are those tickets or tasks which have been flagged as either new features or
    // enhancements to the platform.
    pub fn is_feature(&self) -> bool {
        match self {
            Action::ZIssue(issue) => issue.is_feature(),
            Action::ZTask(task) => task.is_feature(),
        }
    }

    // The custom field 'From a client(:)?' indicates whether an action was requested by an
    // existing paying user of Market Dojo.
    pub fn has_client(&self) -> bool {
        match self {
            Action::ZIssue(issue) => issue.has_client(),
            Action::ZTask(task) => task.has_client(),
        }
    }

    // This is largely necessary because an issue has custom_fields while a task has customfields,
    // according to the Zoho API. We abstract this and make it consistent.
    pub fn custom_fields(&self) -> Option<Vec<CustomField>> {
        match self {
            Action::ZIssue(issue) => Some(
                issue
                    .0
                    .customfields
                    .as_ref()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(CustomField::from)
                    .collect(),
            ),
            Action::ZTask(task) => Some(
                task.0
                    .custom_fields
                    .as_ref()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(CustomField::from)
                    .collect(),
            ),
        }
    }

    // Format the relevant parts of an action into a pipe-delimited string, for writing to a
    // Markdown document; this format is intended to produce a table.
    pub fn display(&self) -> Result<String> {
        let rgx = QUOTES.get_or_try_init(|| -> std::result::Result<Regex, regex::Error> {
            Regex::new("[`'’\"]")
        })?;
        match self {
            Action::ZIssue(issue) => Ok(format!(
                "| [{}] {} | {} | {} |",
                issue.0.key,
                rgx.replace_all(&issue.0.title, ""),
                issue.0.issue_type(),
                issue.0.reported_person
            )),
            Action::ZTask(task) => Ok(format!(
                "| [{}] {} | {} | {} |",
                task.0.key,
                rgx.replace_all(&task.0.name, ""),
                task.0.issue_type(),
                task.0.created_person
            )),
        }
    }
}

// Holds a bug (either a Task or an Issue) and the collection of clients who have raised or
// otherwise requested action on that bug.
pub struct ClientBug {
    clients: Vec<String>,
    bug: Action,
}

// Container for actions, separated into the three categories which are reported on:
// client_bugs: actions which have clients associated,
// features: actions which are either new features or enhancements to the platform,
// others: actions which are neither features nor associated with a client, generally bugs or QA
// concerns but also most infrastructure changes.
pub struct ClassifiedActions {
    client_bugs: Vec<ClientBug>,
    features: Vec<Action>,
    others: Vec<Action>,
}

impl ClassifiedActions {
    // Generator for ClassifiedActions; assumes everything is empty to begin with
    pub fn new() -> ClassifiedActions {
        ClassifiedActions {
            client_bugs: vec![],
            features: vec![],
            others: vec![],
        }
    }

    // Somewhat arbitrarily, this ensures that the actions are sorted consistently. Client bugs
    // are sorted by the number of clients associated, while features and other actions are
    // merely sorted alphabetically.
    pub fn sort(mut self) -> Self {
        self.client_bugs
            .sort_by(|a, b| a.clients.len().cmp(&b.clients.len()));
        self.features.sort_by_key(|a| a.name());
        self.others.sort_by_key(|a| a.name());

        self
    }
}

// Utilises the configuration defined in config.yml to generate a new ZohoClient, for interacting
// with ZohoHorrorshow
pub fn zh_client(project: &Project, config: &Config) -> Result<ZohoClient> {
    let client = ZohoClient::new(&config.zoho_client_id, &config.zoho_client_secret);

    client
        .set_portal(&config.zoho_portal_name)
        .map_err(|e| eyre!("Could not set portal: {}", e))?
        .set_project(&project.name)
        .map_err(|e| eyre!("Could not set project: {}", e))
}

// Given a Vec of either Issues or Tasks, partition them by type
pub fn classify_actions(issues: Vec<Action>) -> ClassifiedActions {
    let mut client_list: ClassifiedActions = ClassifiedActions::new();

    for issue in issues {
        issue.custom_fields().and_then(|cfs| {
            cfs.into_iter()
                .find(|cf| cf.label_name.to_lowercase().contains("from a client"))
                .map(|cf| {
                    cf.value
                        .split(',')
                        .map(std::borrow::ToOwned::to_owned)
                        .collect()
                })
                .map(|clients| {
                    client_list.client_bugs.push(ClientBug {
                        clients,
                        bug: issue.clone(),
                    });
                })
        });

        if issue.is_feature() {
            client_list.features.push(issue.clone())
        };

        if !issue.has_client() && !issue.is_feature() {
            client_list.others.push(issue.clone())
        };
    }

    client_list
}

// Collects multiple sets of actions, generally one of Tasks and one of Issues, collect their
// contents into a single set of ClassifiedActions.
pub fn merge_actions(
    mut issue_list: ClassifiedActions,
    mut task_list: ClassifiedActions,
) -> ClassifiedActions {
    issue_list.client_bugs.append(&mut task_list.client_bugs);
    issue_list.features.append(&mut task_list.features);
    issue_list.others.append(&mut task_list.others);
    issue_list
}

// Assembles a string representing the contents of a Markdown file reporting on the contents of a
// set of ClassifiedActions, for later printing or display.
pub fn write_actions(client_list: ClassifiedActions) -> Result<String> {
    let sorted_tickets = client_list.sort();

    let mut output = String::new();

    let client_bugs = sorted_tickets.client_bugs;

    if client_bugs.is_empty() {
        output.push_str("\nNo client bugs or features to report on this Sprint.\n");
    } else {
        output.push_str(
            "### Client Bugs and Features\n\n| Ticket Name | Ticket Type | Raised By | Clients |\n| --- | --- | --- | --- |",
        );

        for client_bug in &client_bugs {
            // This is infallible - unwrap here is safe.
            write!(
                output,
                "\n{} {} |",
                client_bug.bug.display()?,
                client_bug.clients.join(";")
            )
            .unwrap();
        }
    }

    let features = sorted_tickets.features;

    if features.is_empty() {
        output.push_str("\n\nNo new features or enhancements to report on this Sprint.\n");
    } else {
        output.push_str(
            "\n\n### Features and Enhancements\n\n| Ticket Name | Ticket Type | Raised By |\n| --- | --- | --- |",
        );

        for feature in &features {
            // This is infallible - unwrap here is safe.
            write!(output, "\n{}", feature.display()?).unwrap();
        }
    }

    let others = sorted_tickets.others;

    if others.is_empty() {
        output.push_str("\n\nNo other bugs or QA concerns to report on this Sprint.\n");
    } else {
        output.push_str(
            "\n\n### Other Bugs\n\n| Ticket Name | Ticket Type | Raised By |\n| --- | --- | --- |",
        );

        for other in &others {
            // This is infallible - unwrap here is safe.
            write!(output, "\n{}", other.display()?).unwrap();
        }
    }

    Ok(output)
}
