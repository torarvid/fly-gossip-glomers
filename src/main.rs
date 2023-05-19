mod message;
mod message_receiver;
mod node;
mod repo;

use std::collections::{HashMap, HashSet};

use crate::message_receiver::MessageReceiver;
use crate::node::Node;
use crate::repo::Repo;

struct App {
    this_node: Option<Node>,
    nodes: HashMap<String, Node>,
    topology: HashMap<String, HashSet<String>>,
}

impl Repo for App {
    fn add_nodes(&mut self, this_node: Node, nodes: Vec<Node>) {
        self.this_node = Some(this_node);
        self.nodes = nodes.into_iter().map(|n| (n.id.clone(), n)).collect();
    }

    fn this_node(&mut self) -> &mut Node {
        self.this_node.as_mut().unwrap()
    }

    fn set_topology(&mut self, topology: HashMap<String, HashSet<String>>) {
        self.topology = topology;
    }

    fn neighbors(&self, node_id: &str) -> Vec<&Node> {
        self.topology
            .get(node_id)
            .unwrap_or(&HashSet::new())
            .iter()
            .map(|id| self.nodes.get(id).unwrap().clone())
            .collect()
    }

    fn all_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }
}

fn main() {
    let app = App {
        this_node: None,
        nodes: HashMap::new(),
        topology: HashMap::new(),
    };
    let mut message_receiver = MessageReceiver::new(Box::new(app));
    message_receiver.start_reading();
}
