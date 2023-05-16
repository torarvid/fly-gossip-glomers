use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

impl Message {
    fn get_reply(&self) -> Message {
        let body_type = match &self.body.typ {
            BodyType::Echo { echo } => BodyType::EchoOk { echo: echo.clone() },
            BodyType::Init {
                node_id: _,
                node_ids: _,
            } => BodyType::InitOk,
            _ => panic!("Unknown message type"),
        };
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                typ: body_type,
                msg_id: None,
                in_reply_to: self.body.msg_id,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Body {
    #[serde(flatten)]
    typ: BodyType,

    #[serde(skip_serializing_if = "Option::is_none")]
    msg_id: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<usize>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum BodyType {
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

fn main() {
    let stdin = std::io::stdin();
    let stdin = stdin.lock();

    let deserializer = serde_json::Deserializer::from_reader(stdin);
    let iterator = deserializer.into_iter::<Message>();
    for item in iterator {
        let message: Message = item.unwrap();
        println!("{}", serde_json::to_string(&message.get_reply()).unwrap());
    }
}
