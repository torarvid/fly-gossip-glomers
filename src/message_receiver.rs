use std::sync::atomic::{AtomicUsize, Ordering};

use crate::message::{Body, BodyInit, BodyType, Message};
use crate::node::Node;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct MessageReceiver {}

impl MessageReceiver {
    pub fn start_reading() {
        let stdin = std::io::stdin();
        let stdin = stdin.lock();

        let deserializer = serde_json::Deserializer::from_reader(stdin);
        let iterator = deserializer.into_iter::<Message>();
        for item in iterator {
            let message: Message = item.unwrap();
            println!(
                "{}",
                serde_json::to_string(&MessageReceiver::get_reply(&message)).unwrap()
            );
        }
    }

    fn get_reply(message: &Message) -> Message {
        let body_type = match &message.body.typ {
            BodyType::Echo(echo) => BodyType::EchoOk(echo.to_owned()),
            BodyType::Init(body) => MessageReceiver::on_init(&body),
            _ => panic!("Unknown message type"),
        };
        Message {
            src: message.dest.clone(),
            dest: message.src.clone(),
            body: Body {
                typ: body_type,
                msg_id: Some(MessageReceiver::get_next_msg_id()),
                in_reply_to: message.body.msg_id,
            },
        }
    }

    fn get_next_msg_id() -> usize {
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn on_init(body_init: &BodyInit) -> BodyType {
        let node = Node {
            id: body_init.node_id.clone(),
        };
        BodyType::InitOk
    }
}
