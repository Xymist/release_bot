use crate::errors::*;
use std::rc::Rc;
use crate::task::Task;
use zohohorrorshow::{client::ZohoClient, models::task};

#[derive(Debug, Clone)]
pub struct TaskIterator {
    pub items: <Vec<Task> as IntoIterator>::IntoIter,
    pub last_full: bool,
    pub client: Rc<ZohoClient>,
    pub start_index: usize,
}

impl TaskIterator {
    pub fn new(client: &Rc<ZohoClient>) -> TaskIterator {
        TaskIterator {
            items: Vec::new().into_iter(),
            last_full: true,
            client: client.clone(),
            start_index: 0,
        }
    }

    pub fn try_next(&mut self) -> Result<Option<Task>> {
        if let Some(task) = self.items.next() {
            return Ok(Some(task));
        }

        if !self.last_full {
            return Ok(None);
        }

        let returned_tasks = task::tasks(&self.client.clone())
            .index(&format!("{}", self.start_index))
            .fetch()?;

        self.last_full = match returned_tasks.len() {
            100 => true,
            _ => false,
        };

        self.start_index += returned_tasks.len();

        let tasks: Vec<Task> = returned_tasks.into_iter().map(Task).collect();
        self.items = tasks.into_iter();

        Ok(self.items.next())
    }
}

impl Iterator for TaskIterator {
    type Item = Result<Task>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(val)) => Some(Ok(val)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}
