use std::collections::{HashMap, HashSet};

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::node_data::*;
use crate::{Node, NodeId};

#[derive(Debug)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphEvent {
    AddChild(NodeId, Option<usize>, NodeData),
    RemoveChild(NodeId, NodeId),
    AddNewParent(NodeId, NodeId, NodeData),

    ReplaceData(NodeId, NodeData),

    SetTranslation(NodeId, Vec3),
    SetRotation(NodeId, Quat),
    SetScale(NodeId, f32),
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: HashMap<NodeId, Node>,
    pub root_node_id: NodeId,

    id_generator: IdGenerator,
}
impl Graph {
    pub fn new(data: NodeData) -> Graph {
        let mut nodes = HashMap::new();
        let mut id_generator = IdGenerator::new();
        let root_node_id = Self::add_direct(&mut nodes, &mut id_generator, data);
        Graph {
            nodes,
            root_node_id,

            id_generator,
        }
    }

    fn add_direct(
        nodes: &mut HashMap<NodeId, Node>,
        id_generator: &mut IdGenerator,
        data: NodeData,
    ) -> NodeId {
        let id = id_generator.generate();
        nodes.insert(id, Node::new(id, data));
        id
    }

    pub fn add(&mut self, data: NodeData) -> NodeId {
        Self::add_direct(&mut self.nodes, &mut self.id_generator, data)
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    fn find_all_reachable_nodes_opt(&self, node_id: Option<NodeId>, seen: &mut HashSet<NodeId>) {
        if let Some(node_id) = node_id {
            self.find_all_reachable_nodes(node_id, seen)
        }
    }

    fn find_all_reachable_nodes(&self, node_id: NodeId, seen: &mut HashSet<NodeId>) {
        let node = self.get(node_id).unwrap();
        seen.insert(node.id);
        match &node.data {
            NodeData::Sphere(_) => {}
            NodeData::Cylinder(_) => {}
            NodeData::Torus(_) => {}

            NodeData::Union(Union { children, .. }) => {
                for child_id in children {
                    self.find_all_reachable_nodes(*child_id, seen);
                }
            }
            NodeData::Intersect(Intersect {
                children: (lhs, rhs),
                ..
            }) => {
                self.find_all_reachable_nodes_opt(*lhs, seen);
                self.find_all_reachable_nodes_opt(*rhs, seen);
            }
            NodeData::Subtract(Subtract { children, .. }) => {
                for child_id in children {
                    self.find_all_reachable_nodes(*child_id, seen);
                }
            }

            NodeData::Rgb(Rgb { child, .. }) => {
                self.find_all_reachable_nodes_opt(*child, seen);
            }

            NodeData::Translate(Translate { child, .. }) => {
                self.find_all_reachable_nodes_opt(*child, seen);
            }
            NodeData::Rotate(Rotate { child, .. }) => {
                self.find_all_reachable_nodes_opt(*child, seen);
            }
            NodeData::Scale(Scale { child, .. }) => {
                self.find_all_reachable_nodes_opt(*child, seen);
            }
        }
    }

    fn garbage_collect(&mut self) {
        let all: HashSet<_> = self.nodes.keys().copied().collect();
        let mut seen = HashSet::new();
        self.find_all_reachable_nodes(self.root_node_id, &mut seen);

        for node_id in all.difference(&seen) {
            self.nodes.remove(node_id);
        }
    }

    fn apply_event(&mut self, event: &GraphEvent) -> Option<()> {
        match event {
            GraphEvent::AddChild(parent_id, index, node_data) => {
                let child_id = self.add(node_data.clone());
                self.get_mut(*parent_id)?.data.add_child(*index, child_id);
            }
            GraphEvent::RemoveChild(parent_id, child_id) => {
                self.get_mut(*parent_id)?.data.remove_child(*child_id);
            }
            GraphEvent::AddNewParent(parent_id, child_id, node_data) => {
                let new_parent_id = self.add(node_data.clone());
                self.get_mut(new_parent_id)?.data.add_child(None, *child_id);

                let parent = self.get_mut(*parent_id)?;
                parent.data.replace_child(*child_id, new_parent_id);
            }

            GraphEvent::ReplaceData(node_id, data) => self.get_mut(*node_id)?.data = data.clone(),

            GraphEvent::SetTranslation(node_id, translation) => {
                self.get_mut(*node_id)?.translation = *translation;
            }
            GraphEvent::SetRotation(node_id, rotation) => {
                self.get_mut(*node_id)?.rotation = *rotation;
            }
            GraphEvent::SetScale(node_id, scale) => {
                self.get_mut(*node_id)?.scale = *scale;
            }
        }

        Some(())
    }

    pub fn apply_events(&mut self, events: &[GraphEvent]) {
        for event in events {
            self.apply_event(event)
                .expect("failed to apply event cleanly");
        }
        self.garbage_collect();
    }
}
