use errors::*;
use issue::Issue;
use std::rc::Rc;
use zohohorrorshow::{
    client::ZohoClient,
    models::bug,
};

#[derive(Debug, Clone)]
pub struct IssueIterator {
    pub items: <Vec<Issue> as IntoIterator>::IntoIter,
    pub last_full: bool,
    pub milestones: Vec<String>,
    pub client: Rc<ZohoClient>,
    pub start_index: usize,
}

impl IssueIterator {
    pub fn new(client: &Rc<ZohoClient>, milestone_ids: Vec<String>) -> IssueIterator {
        IssueIterator {
            items: Vec::new().into_iter(),
            last_full: true,
            milestones: milestone_ids,
            client: client.clone(),
            start_index: 0,
        }
    }

    pub fn try_next(&mut self) -> Result<Option<Issue>> {
        if let Some(issue) = self.items.next() {
            return Ok(Some(issue));
        }

        if !self.last_full {
            return Ok(None);
        }

        let returned_tickets = bug::bugs(&self.client.clone())
            .milestone(
                self.milestones
                    .iter()
                    .map(|s| &**s)
                    .collect::<Vec<&str>>()
                    .as_slice(),
            ).sort_column("last_modified_time")
            .sort_order("descending")
            .index(&format!("{}", self.start_index))
            .fetch()?;

        self.last_full = match returned_tickets.len() {
            100 => true,
            _ => false,
        };

        self.start_index += returned_tickets.len();

        let issues: Vec<Issue> = returned_tickets.into_iter().map(Issue).collect();
        self.items = issues.into_iter();

        Ok(self.items.next())
    }
}

impl Iterator for IssueIterator {
    type Item = Result<Issue>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(val)) => Some(Ok(val)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}