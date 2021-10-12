use libangelshark::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct Error {
    pub reason: String,
}

#[derive(Serialize, Debug)]
pub struct Version {
    pub daemon_version: &'static str,
}

#[derive(Deserialize, Debug)]
pub struct Query {
    pub no_cache: Option<bool>,
    pub panicky: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct Request {
    pub acms: Vec<String>,
    pub command: String,
    pub fields: Option<Vec<String>>,
    pub datas: Option<Vec<Vec<String>>>,
}

impl From<Request> for Vec<(String, Message)> {
    fn from(val: Request) -> Self {
        val.acms
            .iter()
            .map(|name| {
                (
                    name.to_owned(),
                    Message {
                        command: val.command.clone(),
                        fields: val.fields.clone(),
                        datas: val.datas.clone(),
                        error: None,
                    },
                )
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub acm: String,
    pub command: String,
    pub error: String,
    pub fields: Vec<String>,
    pub datas: Vec<Vec<String>>,
}

impl From<(String, Message)> for Response {
    fn from(msg: (String, Message)) -> Self {
        let (acm, msg) = msg;
        Self {
            acm,
            command: msg.command,
            fields: msg.fields.unwrap_or_default(),
            datas: msg.datas.unwrap_or_default(),
            error: msg.error.unwrap_or_default(),
        }
    }
}
