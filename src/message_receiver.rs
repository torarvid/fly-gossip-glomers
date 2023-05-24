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

fn now_us() -> u128 {
    let time = std::time::SystemTime::now();
    let since_the_epoch = time
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_micros()
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
        eprintln!("{}: Received message: {:?}", now_us(), message);
        if let Some(body_type) = match message.body.typ.clone() {
            BodyType::Echo(body) => Some(BodyType::EchoOk(body)),
            BodyType::Init(body) => Some(self.on_init(body)),
            BodyType::Generate => Some(BodyType::GenerateOk(BodyGenerate::new())),
            BodyType::Broadcast(body) => self.on_broadcast(body, message.clone()),
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
        eprintln!("{}: Sent message: {:?}", now_us(), message);
    }

    fn get_next_msg_id() -> usize {
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn queue_retrier(&mut self) {
        let msg_queue_lock = Arc::clone(&self.msg_queue);
        thread::spawn(move || {
            let mut retry_number = 0;
            loop {
                let mut msg_vec = Vec::new();
                {
                    if let Ok(msg_queue) = msg_queue_lock.read() {
                        for message in msg_queue.values() {
                            msg_vec.push(message.clone());
                        }
                    }
                }
                for message in msg_vec {
                    MessageReceiver::send_outgoing(&message);
                }
                retry_number += 1;
                thread::sleep(std::time::Duration::from_millis(1000 * retry_number));
            }
        });
    }

    fn on_init(&mut self, body_init: BodyInit) -> BodyType {
        let (node, nodes) = body_init.into();
        self.repo.add_nodes(node, nodes);
        BodyType::InitOk
    }

    fn on_broadcast(&mut self, body_broadcast: BodyBroadcast, source: Message) -> Option<BodyType> {
        eprintln!("{}:   Received broadcast: {:?}", now_us(), body_broadcast);
        let message = Message {
            src: source.dest.clone(),
            dest: source.src.clone(),
            body: Body {
                typ: BodyType::BroadcastOk,
                msg_id: Some(MessageReceiver::get_next_msg_id()),
                in_reply_to: source.body.msg_id,
            },
        };
        MessageReceiver::send_outgoing(&message);

        let node_id: Option<String> = {
            if let Some(node) = self.repo.this_node() {
                if !node.has_message(body_broadcast.message) {
                    node.add_message(body_broadcast.message);
                    Some(node.id())
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(node_id) = node_id {
            let neighbors: Vec<Node> = self.repo.neighbors(&node_id).into_iter().cloned().collect();

            eprintln!("{}:   Broadcasting: {:?}", now_us(), body_broadcast);
            for neighbor_node in neighbors {
                if neighbor_node.id() == source.src || neighbor_node.id() == node_id.to_owned() {
                    continue;
                }
                let msg_id = MessageReceiver::get_next_msg_id();
                let message = Message {
                    src: node_id.to_owned(),
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
                MessageReceiver::send_outgoing(&message);
                if let Ok(mut msg_queue) = self.msg_queue.write() {
                    msg_queue.insert(msg_id, message);
                }
                eprintln!("{}:   Queued message: {:?}", now_us(), msg_id);
            }
            return Some(BodyType::BroadcastOk);
        }
        eprintln!("{}:   Done broadcasting: {:?}", now_us(), body_broadcast);
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
