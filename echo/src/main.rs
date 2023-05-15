use serde_derive::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
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

fn get_reply(message: Message) -> Message {
    let body_type = match message.body.typ {
        BodyType::Echo { echo } => BodyType::EchoOk { echo },
        BodyType::Init {
            node_id: _,
            node_ids: _,
        } => BodyType::InitOk,
        _ => panic!("Unknown message type"),
    };
    Message {
        src: message.dest,
        dest: message.src,
        body: Body {
            typ: body_type,
            msg_id: None,
            in_reply_to: message.body.msg_id,
        },
    }
}

fn main() {
    let stdin = std::io::stdin();
    let stdin = stdin.lock();

    let deserializer = serde_json::Deserializer::from_reader(stdin);
    let iterator = deserializer.into_iter::<Message>();
    for item in iterator {
        let message: Message = item.unwrap();
        let message = get_reply(message);
        println!("{}", serde_json::to_string(&message).unwrap());
    }
}
