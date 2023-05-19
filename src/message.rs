use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

#[derive(Serialize, Deserialize)]
pub struct Body {
    #[serde(flatten)]
    pub typ: BodyType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<usize>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum BodyType {
    Echo(BodyEcho),
    EchoOk(BodyEcho),
    Init(BodyInit),
    InitOk,
    Generate,
    GenerateOk(BodyGenerate),
    Broadcast(BodyBroadcast),
    BroadcastOk,
    Read,
    ReadOk(BodyReadOk),
    Topology(BodyTopology),
    TopologyOk,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BodyEcho {
    pub echo: String,
}

#[derive(Serialize, Deserialize)]
pub struct BodyInit {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BodyGenerate {
    pub id: String,
}

impl BodyGenerate {
    pub fn new() -> Self {
        let id = ulid::Ulid::new().to_string();
        Self { id }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BodyBroadcast {
    pub message: usize,
}

#[derive(Serialize, Deserialize)]
pub struct BodyReadOk {
    pub messages: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct BodyTopology {
    pub topology: HashMap<String, Vec<String>>,
}
