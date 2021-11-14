mod api;

use std::env;

fn main() {
    let api_key = env::var("TOGGL_API_KEY").expect("Need to set TOGGL_API_KEY env var");
    let api_client = api::Api::new(&api_key);
    let workspaces = api_client.workspaces_get_all().unwrap();
    println!("{:?}", api_client.workspaces_projects_all(workspaces[0].id.unwrap()));
    println!("{:?}", api_client.workspaces_tags_all(workspaces[0].id.unwrap()));

    let params = api::ReportsDetailedParams::new("Toggle Oxide".to_string(), 5864726, 1);
    println!("{:?}", api_client.reports_detailed(&params));
}
