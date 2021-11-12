mod api;

use std::env;

fn main() {
    let api_key = env::var("TOGGL_API_KEY").expect("Need to set TOGGL_API_KEY env var");
    let api = api::Api::new(&api_key);
    let workspaces = api.workspaces_get_all().unwrap();
    println!("{:?}", api.workspaces_projects_all(workspaces[0].id.unwrap()));
    println!("{:?}", api.workspaces_tags_all(workspaces[0].id.unwrap()));
}
