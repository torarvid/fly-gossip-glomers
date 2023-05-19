use std::collections::{HashMap, HashSet};

use crate::node::Node;

pub trait Repo {
    fn add_nodes(&mut self, this_node: Node, nodes: Vec<Node>);
    fn this_node(&mut self) -> &mut Node;
    fn set_topology(&mut self, topology: HashMap<String, HashSet<String>>);
    fn neighbors(&self, node_id: &str) -> Vec<&Node>;
    fn all_nodes(&self) -> Vec<&Node>;
}
