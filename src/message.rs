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
