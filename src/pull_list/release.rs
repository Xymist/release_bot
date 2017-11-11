use std::fmt;
use chrono::{Utc, TimeZone, DateTime, offset};

#[derive(Deserialize, Debug, Clone)]
pub struct Release {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub tag_name: Option<String>,
    pub body: Option<String>,
    pub created_at: DateTime<offset::Utc>,
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, published {}",
            self.name.as_ref().unwrap(),
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
            created_at: Utc.ymd(2001, 1, 1).and_hms(0, 0, 0),
        }
    }
}