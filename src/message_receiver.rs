use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::message::{
    Body, BodyBroadcast, BodyGenerate, BodyInit, BodyReadOk, BodyTopology, BodyType, Message,
};
use crate::node::Node;
use crate::repo::Repo;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct MessageReceiver {
    pub repo: Box<dyn Repo>,
}

impl MessageReceiver {
    pub fn start_reading(&mut self) {
        let stdin = std::io::stdin();
        let stdin = stdin.lock();

        let deserializer = serde_json::Deserializer::from_reader(stdin);
        let iterator = deserializer.into_iter::<Message>();
        for item in iterator {
            let message: Message = item.unwrap();
            println!(
                "{}",
                serde_json::to_string(&self.get_reply(&message)).unwrap()
            );
        }
    }

    fn get_reply(&mut self, message: &Message) -> Message {
        let body_type = match &message.body.typ {
            BodyType::Echo(body) => BodyType::EchoOk(body.to_owned()),
            BodyType::Init(body) => self.on_init(&body),
            BodyType::Generate => BodyType::GenerateOk(BodyGenerate::new()),
            BodyType::Broadcast(body) => self.on_broadcast(body),
            BodyType::Read => self.on_read(),
            BodyType::Topology(body) => self.on_topology(body),
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

    fn on_init(&mut self, body_init: &BodyInit) -> BodyType {
        let node = Node::new(body_init.node_id.clone());
        let nodes = body_init
            .node_ids
            .iter()
            .map(|id| Node::new(id.clone()))
            .collect();
        self.repo.add_nodes(node, nodes);
        BodyType::InitOk
    }

    fn on_broadcast(&mut self, body_broadcast: &BodyBroadcast) -> BodyType {
        let node = self.repo.this_node();
        node.add_message(body_broadcast.message);
        BodyType::BroadcastOk
    }

    fn on_read(&mut self) -> BodyType {
        let node = self.repo.this_node();
        let messages: Vec<usize> = node.get_messages().iter().map(|message| *message).collect();
        BodyType::ReadOk(BodyReadOk { messages })
    }

    fn on_topology(&mut self, body_topology: &BodyTopology) -> BodyType {
        let mut topology = HashMap::new();
        for (src, dests) in body_topology.topology.iter() {
            let mut set = HashSet::new();
            for d in dests.iter() {
                set.insert(d.clone());
            }
            topology.insert(src.clone(), set);
        }
        self.repo.set_topology(topology);
        BodyType::TopologyOk
    }
}
