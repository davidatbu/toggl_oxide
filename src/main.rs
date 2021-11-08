use chrono::{Duration, Utc};
mod api;

use api::TimeEntry;
use std::env;

fn main() {
    let api_key = env::var("TOGGL_API_KEY").expect("Need to set TOGGL_API_KEY env var");
    let api = api::Api::new(&api_key);

    let start = Utc::now() - Duration::seconds(500);

    let time_entry = TimeEntry {
        description: Some("This is a test".to_string()),
        wid: None,
        pid: None,
        tid: None,
        billable: None,
        start,
        stop: None,
        duration: 400,
        created_with: Some("Davidat".to_string()),
        tags: None,
        duronly: None,
        at: None,
    };

    println!("{:?}", api.time_entry_create(&time_entry));
}
