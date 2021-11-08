use chrono::{DateTime, Utc};
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::{self, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json;

const API_URL: &str = "https://api.track.toggl.com/api/v8";

#[derive(Debug)]
pub struct ServerError {
    status_code: StatusCode,
    text: Option<String>,
    parsed_json: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ParsingError {
    text: String,
    err: Option<serde_json::error::Error>,
}

/// Encapsulates all errors possible when making a request
#[derive(Debug)]
pub enum ApiError {
    /// When the network fails
    Network(reqwest::Error),

    /// An error message from server. Toggl's eror messages are always an array of strings.
    Server(ServerError),

    /// Couldn't parse server resposne
    Parsing(ParsingError),
}

/// Response from server, or network error, or JSON parsing error
pub type ApiResult<Blob> = Result<Blob, ApiError>;

/// Trait to DRY up code to make a request, parse the JSON, and return an ApiError of the
/// appropriate type if necessary
trait ConsolidateApiErrors {
    fn send_and_get_json<Blob>(self) -> ApiResult<Blob>
    where
        Blob: DeserializeOwned;
}

/// Response from server, whether error or correct
#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResponse<Blob> {
    Blob(Blob),
    ServerError(Vec<String>),
}

impl ConsolidateApiErrors for RequestBuilder {
    fn send_and_get_json<Blob>(self) -> ApiResult<Blob>
    where
        Blob: DeserializeOwned,
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
                        return match serde_json::from_str::<ApiResponse<Blob>>(&txt) {
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

// The main Api object we export
pub struct Api<'a> {
    api_key: &'a str,
    client: Client,
}

// https://github.com/toggl/toggl_api_docs/blob/master/chapters/time_entries.md#time-entries
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeEntry {
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
    pub created_with: Option<String>,

    // a list of tag names ( not required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    // should Toggl show the start and stop time of this time entry? ( not required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duronly: Option<bool>,

    // ONLY sent in response. I hope this doesn't mess up requests.
    // Timestamp that is sent in the response, indicates the time item was last update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<DateTime<Utc>>,
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

impl<'a> Api<'a> {
    pub fn new(api_key: &'a str) -> Api {
        Api {
            api_key,
            client: Client::new(),
        }
    }

    pub fn time_entry_create(&self, time_entry: &TimeEntry) -> ApiResult<TimeEntryResponse> {
        let endpoint = API_URL.to_owned() + "/time_entries";
        println!("Requesting: {}", endpoint);
        let result = self
            .client
            .post(endpoint)
            .add_api_key(self)
            .json(&TimeEntryRequest {
                time_entry: time_entry.clone(),
            })
            .send_and_get_json::<TimeEntryResponse>();
        return result;
    }
}
