use chrono::{DateTime, Utc};
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::{self, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json;

const API_URL: &str = "https://api.track.toggl.com/api/v8";
const REPORTS_API_URL: &str = "https://api.track.toggl.com/reports/api/v2/details";

#[derive(Debug)]
pub struct ServerError<ErrorShape: DeserializeOwned> {
    status_code: StatusCode,
    text: Option<String>,
    parsed_json: Option<ErrorShape>,
}

#[derive(Debug)]
pub struct ParsingError {
    text: String,
    err: Option<serde_json::error::Error>,
}

/// Encapsulates all errors possible when making a request
#[derive(Debug)]
pub enum ApiError<ErrorShape: DeserializeOwned> {
    /// When the network fails
    Network(reqwest::Error),

    /// An error message from server. Toggl's eror messages are always an array of strings.
    Server(ServerError<ErrorShape>),

    /// Couldn't parse server resposne
    Parsing(ParsingError),
}

/// Response from server, or network error, or JSON parsing error
pub type ApiResult<ErrorShape> = Result<Blob, ApiError<ErrorShape>>;

/// Trait to DRY up code to make a request, parse the JSON, and return an ApiError of the
/// appropriate type if necessary
trait ConsolidateApiErrors {
    fn get_json<Blob, ErrorShape>(self) -> ApiResult<Blob, ErrorShape>
    where
        Blob: DeserializeOwned,
        ErrorShape: DeserializeOwned;
}

/// Response from server, whether error or correct
#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResponse<Blob, ErrorShape> {
    Blob(Blob),
    ServerError(ErrorShape),
}

impl ConsolidateApiErrors for RequestBuilder {
    fn get_json<Blob, ErrorShape>(self) -> ApiResult<Blob, ErrorShape>
    where
        Blob: DeserializeOwned,
        ErrorShape: DeserializeOwned,
    {
        return match self.send() {
            Err(err) => Err(ApiError::Network(err)),
            Ok(resp) => {
                if resp.status() != 200 {
                    return Err(ApiError::Server(ServerError {
                        parsed_json: None,
                        status_code: resp.status(),
                        text: resp.text().ok(),
                    }));
                }
                let status_code = resp.status().clone();
                return match resp.text() {
                    Ok(txt) => {
                        return match serde_json::from_str::<ApiResponse<Blob, ErrorShape>>(&txt) {
                            Ok(json) => match json {
                                ApiResponse::ServerError(errors) => {
                                    Err(ApiError::Server(ServerError {
                                        parsed_json: Some(errors),
                                        status_code,
                                        text: None,
                                    }))
                                }
                                ApiResponse::Blob(blob) => Ok(blob),
                            },
                            Err(err) => Err(ApiError::Parsing(ParsingError {
                                text: txt,
                                err: Some(err),
                            })),
                        }
                    }
                    Err(_) => Err(ApiError::Parsing(ParsingError {
                        text: "Couldn't fetch response text.".to_string(),
                        err: None,
                    })),
                };
            }
        };
    }
}

// A trait to add .add_api_key to reqwest::Client
trait AddApiKey {
    fn add_api_key(self, api: &Api) -> Self;
}

impl AddApiKey for RequestBuilder {
    fn add_api_key(self, api: &Api) -> Self {
        return self.basic_auth(api.api_key, Some("api_token"));
    }
}

// https://github.com/toggl/toggl_api_docs/blob/master/chapters/time_entries.md#time-entries
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeEntry {
    /// The id field is not necessary when creating a time entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    // strongly suggested to be used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    // workspace ID ( required if pid or tid not supplied)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wid: Option<i64>,

    // project ID ( not required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<i64>,

    // task ID ( not required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tid: Option<i64>,

    // not required, default false, available for pro workspaces
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billable: Option<bool>,

    // time entry start time ( required, ISO 8601 date and time)
    pub start: DateTime<Utc>,

    // time entry stop time ( not required, ISO 8601 date and time)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<DateTime<Utc>>,

    // time entry duration in seconds. If the time entry is currently running,
    // the duration attribute contains a negative value, denoting the start
    // of the time entry in seconds since epoch (Jan 1 1970). The correct
    // duration can be calculated as current_time + duration, where
    // current_time is the current time in seconds since epoch.
    pub duration: i64,

    /// the name of your client app ( required). It's an Option<> because the response to some API
    /// endpoints doesn't contain this field, and I want to avoid creating a whole other struct
    /// just without this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_with: Option<String>,

    // a list of tag names ( not required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    // should Toggl show the start and stop time of this time entry? ( not required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duronly: Option<bool>,

    /// ONLY sent in response. I hope this doesn't mess up requests.
    /// Timestamp that is sent in the response, indicates the time item was last update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TotalCurrency {
    currency: String,
    amount: f64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Report<Data> {
    total_grand: i64,
    total_billable: i64,
    total_count: i64,
    per_page: i64,
    total_currencies: Vec<TotalCurrency>,
    data: Data
}

/*
 * The JSON schema for the time entries in the reports/ endpoint.
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct ReportTimeEntry {
    /// time entry id
    id: i64,

    /// project id
    pid: Option<i64>,

    /// project name for which the time entry was recorded
    project: Option<i64>,

    /// client name for which the time entry was recorded
    client: Option<String>,

    /// task id
    tid: Option<i64>,

    /// task name for which the time entry was recorded
    task: Option<String>,

    /// user id whose time entry it is
    uid: i64,

    /// full name of the user whose time entry it is
    user: String,

    /// time entry description
    description: Option<String>,

    /// start time of the time entry in ISO 8601 date and time format (YYYY-MM-DDTHH:MM:SS)
    start: DateTime<Utc>,

    /// end time of the time entry in ISO 8601 date and time format (YYYY-MM-DDTHH:MM:SS)
    end: Option<DateTime<Utc>>,

    /// time entry duration in milliseconds
    dur: i64,

    /// last time the time entry was updated in ISO 8601 date and time format (YYYY-MM-DDTHH:MM:SS)
    updated: i64,

    /// if the stop time is saved on the time entry, depends on user's personal settings.
    use_stop: bool,

    /// boolean, if the time entry was billable or not
    is_billable: bool,

    /// billed amount
    billable: i64,

    /// billable amount currency
    cur: String,

    /// array of tag names, which assigned for the time entry
    tags: Vec<String>,

    /// Undocumented on Github API docs.
    project_color: String,

    /// Undocumented on Github API docs.
    project_hex_color: Option<String>,
}

/// This is the structure of the json to POST
#[derive(Serialize, Deserialize, Debug)]
struct TimeEntryRequest {
    time_entry: TimeEntry,
}

/// This is the structure of the json response
#[derive(Serialize, Deserialize, Debug)]
pub struct TimeEntryResponse {
    data: TimeEntry,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Workspace {
    /// The id field is not necessary when creating a workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    /// the name of the workspace
    pub name: String,

    /// If it's a pro workspace or not. Shows if someone is paying for the workspace or not
    pub premium: bool,

    /// shows whether currently requesting user has admin access to the workspace
    pub admin: bool,

    /// default hourly rate for workspace, won't be shown to non-admins if the only_admins_see_billable_rates flag is set to true
    pub default_hourly_rate: f64,

    /// default currency for workspace
    pub default_currency: String,

    /// whether only the admins can create projects or everybody
    pub only_admins_may_create_projects: bool,

    /// whether only the admins can see billable rates or everybody
    pub only_admins_see_billable_rates: bool,

    /// type of rounding
    pub rounding: i64,

    /// round up to nearest minute
    pub rounding_minutes: i64,

    /// timestamp that indicates the time workspace was last updated
    pub at: DateTime<Utc>,

    /// URL pointing to the logo [if set, otherwise omited]
    pub logo_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    /// The id field is not necessary when creating a workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    /// The name of the tag (unique in workspace)
    pub name: String,

    /// workspace ID, where the tag will be used
    pub wid: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    /// The id field is not necessary when creating a workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    /// The name of the project (required, unique for client and workspace)
    name: String,

    /// workspace ID, where the project will be saved (required)
    wid: i64,

    /// client ID (not required)
    cid: Option<i64>,

    /// whether the project is archived or not (by default true)
    active: bool,

    /// whether project is accessible for only project users or for all workspace users (default true)
    is_private: bool,

    /// whether the project can be used as a template (not required)
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<bool>,

    /// id of the template project used on current project's creation
    template_id: Option<i64>,

    /// whether the project is billable or not (default true, available only for pro workspaces)
    billable: bool,

    /// whether the estimated hours are automatically calculated based on task estimations or manually fixed based on the value of 'estimated_hours' (default false, not required, premium functionality)
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_estimates: Option<bool>,

    /// if auto_estimates is true then the sum of task estimations is returned, otherwise user inserted hours (not required, premium functionality)
    #[serde(skip_serializing_if = "Option::is_none")]
    estimated_hours: Option<i64>,

    /// timestamp that is sent in the response for PUT, indicates the time task was last updated (read-only)
    at: DateTime<Utc>,

    /// id of the color selected for the project
    color: String,

    /// hourly rate of the project (not required, premium functionality)
    #[serde(skip_serializing_if = "Option::is_none")]
    rate: Option<f64>,

    /// timestamp indicating when the project was created (UTC time), read-only
    created_at: DateTime<Utc>,
}

/// The main Api object
pub struct Api<'a> {
    api_key: &'a str,
    client: Client,
}

#[derive(Serialize, Debug, Default)]
pub struct ReportsParams {
    // Required. The name of your application or your email address so we can get in touch in case you're doing something wrong.
    user_agent: String,
    // Required. The workspace whose data you want to access.
    workspace_id: i64,

    /// ISO 8601 date (YYYY-MM-DD) format. Defaults to today - 6 days.
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<DateTime<Utc>>,

    /// ISO 8601 date (YYYY-MM-DD) format. Note: Maximum date span (until - since) is one year.
    /// Defaults to today, unless since is in future or more than year ago, in this case until is since + 6 days.
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime<Utc>>,

    /// "yes", "no", or "both". Defaults to "both".
    #[serde(skip_serializing_if = "Option::is_none")]
    billable: Option<String>,

    /// A list of client IDs separated by a comma. Use "0" if you want to filter out time entries without a client.
    #[serde(skip_serializing_if = "Option::is_none")]
    client_ids: Option<Vec<i64>>,

    /// A list of project IDs separated by a comma. Use "0" if you want to filter out time entries without a project.
    #[serde(skip_serializing_if = "Option::is_none")]
    project_ids: Option<Vec<i64>>,

    /// A list of user IDs separated by a comma.
    #[serde(skip_serializing_if = "Option::is_none")]
    user_ids: Option<Vec<i64>>,

    /// A list of group IDs separated by a comma. This limits provided user_ids to the members of the given groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    members_of_group_ids: Option<Vec<i64>>,

    /// A list of group IDs separated by a comma. This extends provided user_ids with the members of the given groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    or_members_of_group_ids: Option<Vec<i64>>,

    /// A list of tag IDs separated by a comma. Use "0" if you want to filter out time entries without a tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    tag_ids: Option<Vec<i64>>,

    /// A list of task IDs separated by a comma. Use "0" if you want to filter out time entries without a task.
    #[serde(skip_serializing_if = "Option::is_none")]
    task_ids: Option<Vec<i64>>,

    /// A list of time entry IDs separated by a comma.
    #[serde(skip_serializing_if = "Option::is_none")]
    time_entry_ids: Option<Vec<i64>>,

    /// Matches against time entry descriptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// "true" or "false". Filters out the time entries which do not have a description (literally "(no description)").
    #[serde(skip_serializing_if = "Option::is_none")]
    without_description: Option<bool>,

    /// For detailed reports: "date", "description", "duration", or "user"
    /// For summary reports: "title", "duration", or "amount"
    /// For weekly reports: "title", "day1", "day2", "day3", "day4", "day5", "day6", "day7", or "week_total"
    #[serde(skip_serializing_if = "Option::is_none")]
    order_field: Option<String>,

    /// "on" for descending, or "off" for ascending order.
    #[serde(skip_serializing_if = "Option::is_none")]
    order_desc: Option<String>,

    /// "on" or "off". Defaults to "off".
    #[serde(skip_serializing_if = "Option::is_none")]
    distinct_rates: Option<String>,

    /// "on" or "off". Defaults to "off". Rounds time according to workspace settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    rounding: Option<String>,

    /// "decimal" or "minutes". Defaults to "minutes". Determines whether to display hours as a decimal number or with minutes.
    #[serde(skip_serializing_if = "Option::is_none")]
    display_hours: Option<String>,
}

// We use serde here to make it easier to build the URL
impl ReportsParams {
    pub fn new(user_agent: String, workspace_id: i64) -> Self {
        Self {
            user_agent,
            workspace_id,
            ..Default::default()
        }
    }
}

// We use serde here to make it easier to build the URL
#[derive(Serialize, Debug)]
struct ReportsDetailedParams {
    #[serde(flatten)]
    reports_params: ReportsParams,
    total_count: i64,
    per_page: i64,
}

impl ReportsDetailedParams {
    pub fn new(user_agent: String, workspace_id: i64, total_count: i64, per_page: i64) -> Self {
        Self {
            reports_params: ReportsParams::new(user_agent, workspace_id),
            total_count,
            per_page,
        }
    }

    pub fn to_url(&self) -> Url {
        let mut url = Url::parse(REPORTS_API_URL).unwrap();
        let hi = url.query_pairs_mut();
        let json = serde_json::to_value(self).unwrap();
        if let serde_json::Value::Object(map) = json {
            for (key, wrapped_val) in map.into_iter() {
                if serde_json::Value::Null == wrapped_val {
                    continue;
                };
                match wrapped_val {
                    serde_json::Value::Bool(val) => &val.to_string(),
                    serde_json::Value::Number(val) => &val.to_string(),
                    serde_json::Value::String(val) => &val,
                    serde_json::Value::Array(val) => &val
                        .into_iter()
                        .map(|x| x.as_str().unwrap())
                        .collect::<Vec<&str>>()
                        .join(","),
                    serde_json::Value::Object(val) => {
                        panic!("Key {} had unexpcted val {:?}", key, val)
                    }
                    serde_json::Value::Null => panic!("Shouldn't happen"),
                };
            }
        } else {
            panic!("unexpected val: {:?}", json)
        }
        return url;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct _ReportsErrorJson {
    message: String,
    tip: String,
    code: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReportsErrorJson {
    error: _ReportsErrorJson,
}

type DefaultErrorJson = Vec<String>;

impl<'a> Api<'a> {
    pub fn new(api_key: &'a str) -> Api {
        Api {
            api_key,
            client: Client::new(),
        }
    }

    fn post_and_get_json<
        BodyJson: Serialize,
        RespJson: DeserializeOwned,
        ErrorShape: DeserializeOwned,
    >(
        &self,
        endpoint: &str,
        body: &BodyJson,
    ) -> ApiResult<RespJson, ErrorShape> {
        println!("Requesting: {}", endpoint);
        let result = self
            .client
            .post(endpoint)
            .add_api_key(self)
            .json(body)
            .get_json();
        return result;
    }

    /// Create a time entry. Look at `TimeEntry`'s documentation for fields that are required.
    pub fn time_entry_create(
        &self,
        time_entry: &TimeEntry,
    ) -> ApiResult<TimeEntryResponse, DefaultErrorJson> {
        let endpoint = API_URL.to_owned() + "/time_entries";
        println!("Requesting: {}", endpoint);
        let result = self.post_and_get_json(
            &endpoint,
            &TimeEntryRequest {
                time_entry: time_entry.clone(),
            },
        );
        return result;
    }

    /// Get workspaces
    pub fn workspaces_get_all(&self) -> ApiResult<Vec<Workspace>, DefaultErrorJson> {
        let endpoint = API_URL.to_owned() + "/workspaces";
        println!("Requesting: {}", endpoint);
        let result = self.client.get(endpoint).add_api_key(self).get_json();
        return result;
    }

    /// Get workspace tags
    pub fn workspaces_tags_all(&self, wid: i64) -> ApiResult<Vec<Tag>, DefaultErrorJson> {
        let endpoint = API_URL.to_owned() + "/workspaces/" + &wid.to_string() + "/tags";
        println!("Requesting: {}", endpoint);
        let result = self.client.get(endpoint).add_api_key(self).get_json();
        return result;
    }

    /// Get workspace projects
    pub fn workspaces_projects_all(&self, wid: i64) -> ApiResult<Vec<Project>, DefaultErrorJson> {
        let endpoint = API_URL.to_owned() + "/workspaces/" + &wid.to_string() + "/projects";
        println!("Requesting: {}", endpoint);
        let result = self.client.get(endpoint).add_api_key(self).get_json();
        return result;
    }

    pub fn get_reports_detailed(
        &self,
        params: &ReportsDetailedParams,
    ) -> ApiResult<Vec<ReportTimeEntry>, ReportsErrorJson> {
    }
}
