use chrono::{offset, DateTime, TimeZone, Utc};
use serde_derive::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug, Clone)]
pub struct Release {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub tag_name: Option<String>,
    pub body: Option<String>,
    pub created_at: DateTime<offset::Utc>,
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, published {}",
            self.name.as_ref().ok_or(fmt::Error)?,
            self.created_at
        )
    }
}

impl Default for Release {
    fn default() -> Release {
        Release {
            id: None,
            name: Some(String::from("First Commit")),
            tag_name: None,
            body: None,
            created_at: Utc.with_ymd_and_hms(2001, 1, 1, 0, 0, 0).unwrap(),
        }
    }
}
