use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{node_data::*, Transform};
use crate::{Node, NodeId};

#[derive(Debug, Serialize, Deserialize)]
struct IdGenerator {
    last_id: NodeId,
    returned_ids: HashSet<NodeId>,
}
impl IdGenerator {
    fn new() -> IdGenerator {
        IdGenerator {
            last_id: NodeId(0),
            returned_ids: HashSet::new(),
        }
    }

    pub fn generate(&mut self) -> NodeId {
        if !self.returned_ids.is_empty() {
            let id = *self.returned_ids.iter().next().unwrap();
            self.returned_ids.remove(&id);
            id
        } else {
            let id = self.last_id;
            self.last_id.0 += 1;
            id
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    root_node_id: Option<NodeId>,

    id_generator: IdGenerator,
}

impl Graph {
    pub fn new_authoritative() -> Graph {
        Graph {
            nodes: HashMap::new(),
            root_node_id: None,
            id_generator: IdGenerator::new(),
        }
    }

    pub fn add(&mut self, data: NodeData, transform: Transform) -> NodeId {
        let id = self.id_generator.generate();
        let node = Node {
            id,
            rgb: Node::DEFAULT_COLOUR,
            transform,
            data,
            children: vec![],
        };
        self.nodes.insert(id, node.clone());
        id
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    pub fn root_node_id(&self) -> Option<NodeId> {
        self.root_node_id
    }
}
