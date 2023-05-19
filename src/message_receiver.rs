use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::{println, thread};

use crate::message::{
    Body, BodyBroadcast, BodyGenerate, BodyInit, BodyReadOk, BodyTopology, BodyType, Message,
};
use crate::node::Node;
use crate::repo::Repo;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct MessageReceiver {
    repo: Box<dyn Repo>,
    msg_queue: Arc<RwLock<HashMap<usize, Message>>>,
}

impl MessageReceiver {
    pub fn new(repo: Box<dyn Repo>) -> Self {
        let mut instance = Self {
            repo,
            msg_queue: Arc::new(RwLock::new(HashMap::new())),
        };
        instance.queue_retrier();
        instance
    }

    pub fn start_reading(&mut self) {
        let stdin = std::io::stdin();
        let stdin = stdin.lock();

        let deserializer = serde_json::Deserializer::from_reader(stdin);
        let iterator = deserializer.into_iter::<Message>();
        for item in iterator {
            let message: Message = item.unwrap();
            self.process_incoming(&message);
        }
    }

    fn process_incoming(&mut self, message: &Message) {
        let body_type = match &message.body.typ {
            BodyType::Echo(body) => Some(BodyType::EchoOk(body.to_owned())),
            BodyType::Init(body) => Some(self.on_init(&body)),
            BodyType::Generate => Some(BodyType::GenerateOk(BodyGenerate::new())),
            BodyType::Broadcast(body) => Some(self.on_broadcast(body)),
            BodyType::BroadcastOk => self.on_broadcast_ok(&message.body),
            BodyType::Read => Some(self.on_read()),
            BodyType::Topology(body) => Some(self.on_topology(body)),
            _ => panic!("Unknown message type {:?}", message.body.typ),
        };
        if let Some(body_type) = body_type {
            let message = Message {
                src: message.dest.clone(),
                dest: message.src.clone(),
                body: Body {
                    typ: body_type,
                    msg_id: Some(MessageReceiver::get_next_msg_id()),
                    in_reply_to: message.body.msg_id,
                },
            };
            MessageReceiver::send_outgoing(&message);
        }
    }

    fn send_outgoing(message: &Message) {
        println!("{}", serde_json::to_string(message).unwrap());
    }

    fn get_next_msg_id() -> usize {
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn queue_retrier(&mut self) {
        let msg_queue = Arc::clone(&self.msg_queue);
        thread::spawn(move || loop {
            let r = msg_queue.read().unwrap();
            for (_msg_id, message) in r.iter() {
                MessageReceiver::send_outgoing(&message);
            }
            thread::sleep(std::time::Duration::from_millis(100));
        });
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
        let node_id = node.id.clone();
        if !node.has_message(body_broadcast.message) {
            node.add_message(body_broadcast.message);
            for neighbor_node in self.repo.all_nodes() {
                let message = Message {
                    src: node_id.clone(),
                    dest: neighbor_node.id.clone(),
                    body: Body {
                        typ: BodyType::Broadcast(BodyBroadcast {
                            message: body_broadcast.message.clone(),
                        }),
                        msg_id: Some(MessageReceiver::get_next_msg_id()),
                        in_reply_to: None,
                    },
                };
                // Ensure that the message is sent at least once
                self.msg_queue
                    .write()
                    .unwrap()
                    .insert(message.body.msg_id.unwrap(), message.to_owned());
            }
        }
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

    fn on_broadcast_ok(&self, body: &Body) -> Option<BodyType> {
        if let Some(msg_id) = body.in_reply_to {
            self.msg_queue.write().unwrap().remove(&msg_id);
        }
        None
    }
}
