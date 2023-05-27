use std::collections::HashMap;

use crate::node::Node;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

impl Message {
    pub fn response(self, body_type: BodyType, next_id: usize) -> Self {
        Message {
            src: self.dest,
            dest: self.src,
            body: Body {
                typ: body_type,
                msg_id: Some(next_id),
                in_reply_to: self.body.msg_id,
            },
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Body {
    #[serde(flatten)]
    pub typ: BodyType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BodyEcho {
    pub echo: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BodyInit {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

impl From<BodyInit> for (Node, Vec<Node>) {
    fn from(body_init: BodyInit) -> Self {
        (
            Node::new(body_init.node_id),
            body_init.node_ids.into_iter().map(Node::new).collect(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BodyGenerate {
    pub id: String,
}

impl BodyGenerate {
    pub fn new() -> Self {
        let id = ulid::Ulid::new().to_string();
        Self { id }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BodyBroadcast {
    pub message: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BodyReadOk {
    pub messages: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BodyTopology {
    pub topology: HashMap<String, Vec<String>>,
}
