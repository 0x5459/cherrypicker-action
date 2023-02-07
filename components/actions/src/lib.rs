use std::{env, fmt, fs, io, str::FromStr};

use logit::Logit;
use octocrab::models::events::{payload::EventPayload, EventType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    /**
     * Webhook payload object that triggered the workflow
     */
    pub payload: EventPayload,

    pub event_name: EventType,
    pub sha: String,
    pub r#ref: String,
    pub workflow: String,
    pub action: String,
    pub actor: String,
    pub job: String,
    pub run_number: usize,
    pub run_id: usize,
    pub api_url: String,
    pub server_url: String,
    pub graphql_url: String,
}
impl Context {
    pub fn from_env() -> Self {
        let payload: EventPayload = env::var("GITHUB_EVENT_PATH")
            .ok()
            .and_then(|p| {
                fs::File::open(&p)
                    .with_logit_warn(|| format!("open github event file {}", p))
                    .ok()
                    .and_then(|f| {
                        serde_json::from_reader(io::BufReader::new(f))
                            .with_logit_warn(|| format!("deserialize github event file {}", p))
                            .ok()
                    })
            })
            .unwrap_or_else(|| EventPayload::UnknownEvent(Box::new(serde_json::Value::Null)));

        Self {
            payload,
            event_name: env::var("GITHUB_EVENT_NAME")
                .ok()
                .map(|x| format!("\"{}\"", x))
                .and_then(|x| serde_json::from_str(&x).ok())
                .unwrap_or_else(|| EventType::UnknownEvent("undefined".to_string())),
            sha: env::var("GITHUB_SHA")
                .ok()
                .unwrap_or_else(|| "".to_string()),
            r#ref: env::var("GITHUB_REF")
                .ok()
                .unwrap_or_else(|| "".to_string()),
            workflow: env::var("GITHUB_WORKFLOW")
                .ok()
                .unwrap_or_else(|| "".to_string()),
            action: env::var("GITHUB_ACTION")
                .ok()
                .unwrap_or_else(|| "".to_string()),
            actor: env::var("GITHUB_ACTOR")
                .ok()
                .unwrap_or_else(|| "".to_string()),
            job: env::var("GITHUB_JOB")
                .ok()
                .unwrap_or_else(|| "".to_string()),
            run_number: env::var("GITHUB_RUN_NUMBER")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(10),
            run_id: env::var("GITHUB_RUN_ID")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(10),
            api_url: env::var("GITHUB_API_URL")
                .ok()
                .unwrap_or_else(|| "https://api.github.com".to_string()),
            server_url: env::var("GITHUB_SERVER_URL")
                .ok()
                .unwrap_or_else(|| "https://github.com".to_string()),
            graphql_url: env::var("GITHUB_GRAPHQL_URL")
                .ok()
                .unwrap_or_else(|| "https://api.github.com/graphql".to_string()),
        }
    }
}

pub fn get_input<T: FromStr>(name: impl AsRef<str>) -> Option<T> {
    let k = format!("INPUT_{}", name.as_ref().replace(' ', "_").to_uppercase());
    env::var(&k).ok().and_then(|x| x.parse().ok())
}

pub fn get_multiline_input(name: impl AsRef<str>) -> Vec<String> {
    get_input(name)
        .map(|v: String| v.lines().map(|x| x.to_string()).collect())
        .unwrap_or_default()
}

#[derive(Debug)]
pub struct StringError(String);

impl std::error::Error for StringError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}

pub fn get_input_required<T: FromStr>(name: impl AsRef<str>) -> Result<T, StringError> {
    get_input(name.as_ref()).ok_or_else(|| {
        StringError(format!(
            "Input required and not supplied: {}",
            name.as_ref()
        ))
    })
}
