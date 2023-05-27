use std::collections::HashMap;
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
            self.process_incoming(message);
        }
    }

    fn process_incoming(&mut self, message: Message) {
        if let Some(body_type) = match message.body.typ.clone() {
            BodyType::Echo(body) => Some(BodyType::EchoOk(body)),
            BodyType::Init(body) => Some(self.on_init(body)),
            BodyType::Generate => Some(BodyType::GenerateOk(BodyGenerate::new())),
            BodyType::Broadcast(body) => self.on_broadcast(body),
            BodyType::BroadcastOk => self.on_broadcast_ok(&message.body),
            BodyType::Read => self.on_read(),
            BodyType::Topology(body) => Some(self.on_topology(body)),
            _ => {
                eprintln!("Unknown message type {:?}", message.body.typ);
                None
            }
        } {
            let response = message.response(body_type, MessageReceiver::get_next_msg_id());
            MessageReceiver::send_outgoing(&response);
        }
    }

    fn send_outgoing(message: &Message) {
        println!("{}", serde_json::to_string(message).unwrap());
    }

    fn get_next_msg_id() -> usize {
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn queue_retrier(&mut self) {
        let msg_queue_lock = Arc::clone(&self.msg_queue);
        thread::spawn(move || loop {
            if let Ok(msq_queue) = msg_queue_lock.read() {
                for (_msg_id, message) in msq_queue.iter() {
                    MessageReceiver::send_outgoing(message);
                }
            };
            thread::sleep(std::time::Duration::from_millis(100));
        });
    }

    fn on_init(&mut self, body_init: BodyInit) -> BodyType {
        let (node, nodes) = body_init.into();
        self.repo.add_nodes(node, nodes);
        BodyType::InitOk
    }

    fn on_broadcast(&mut self, body_broadcast: BodyBroadcast) -> Option<BodyType> {
        let neighbors: Vec<Node> = self.repo.all_nodes().into_iter().cloned().collect();

        if let Some(node) = self.repo.this_node() {
            if !node.has_message(body_broadcast.message) {
                node.add_message(body_broadcast.message);
                for neighbor_node in neighbors {
                    let msg_id = MessageReceiver::get_next_msg_id();
                    let message = Message {
                        src: node.id(),
                        dest: neighbor_node.id(),
                        body: Body {
                            typ: BodyType::Broadcast(BodyBroadcast {
                                message: body_broadcast.message,
                            }),
                            msg_id: Some(msg_id),
                            in_reply_to: None,
                        },
                    };
                    // Ensure that the message is sent at least once
                    if let Ok(mut msg_queue) = self.msg_queue.write() {
                        msg_queue.insert(msg_id, message);
                    }
                }
            }
            return Some(BodyType::BroadcastOk);
        }
        None
    }

    fn on_read(&mut self) -> Option<BodyType> {
        if let Some(node) = self.repo.this_node() {
            let messages: Vec<usize> = node.get_messages().iter().copied().collect();
            return Some(BodyType::ReadOk(BodyReadOk { messages }));
        }
        None
    }

    fn on_topology(&mut self, body_topology: BodyTopology) -> BodyType {
        self.repo.set_topology(body_topology.topology);
        BodyType::TopologyOk
    }

    fn on_broadcast_ok(&self, body: &Body) -> Option<BodyType> {
        if let Some(msg_id) = body.in_reply_to {
            if let Ok(mut msg_queue) = self.msg_queue.write() {
                msg_queue.remove(&msg_id);
            }
        }
        None
    }
}
