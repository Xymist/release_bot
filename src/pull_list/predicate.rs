use crate::errors::*;
use crate::pull_list::pull::Pull;
use crate::pull_list::release::Release;
use chrono::NaiveDate;

pub struct Predicate {
    since: Option<NaiveDate>,
}

impl Predicate {
    pub fn from_release(release: &Release) -> Result<Predicate> {
        Ok(Predicate {
            since: Some(release.created_at.date().naive_utc()),
        })
    }

    pub fn test(&self, pull: &Pull) -> bool {
        let pull_closed = pull.closed_at.date().naive_utc();
        self.since.map(|v| pull_closed > v).unwrap_or(true)
    }
}
