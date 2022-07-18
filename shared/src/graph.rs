use std::collections::{HashMap, HashSet};

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::{node_data::*, Transform};
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

    ApplyDiff(NodeId, NodeDataDiff),

    SetColour(NodeId, (f32, f32, f32)),
    SetTranslation(NodeId, Vec3),
    SetRotation(NodeId, Quat),
    SetScale(NodeId, f32),
}

#[derive(Debug)]
pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    root_node_id: Option<NodeId>,

    id_generator: Option<IdGenerator>,
}
impl Graph {
    pub fn new_authoritative(data: NodeData) -> Graph {
        let mut nodes = HashMap::new();
        let mut id_generator = IdGenerator::new();
        let root_node_id = Some(Self::add_direct(&mut nodes, &mut id_generator, data));
        Graph {
            nodes,
            root_node_id,
            id_generator: Some(id_generator),
        }
    }

    pub fn new_client() -> Graph {
        Graph {
            nodes: Default::default(),
            root_node_id: None,
            id_generator: None,
        }
    }

    pub fn from_components(nodes: HashMap<NodeId, Node>, root_node_id: Option<NodeId>) -> Graph {
        Graph {
            nodes,
            root_node_id,
            id_generator: None,
        }
    }

    pub fn to_components(&self) -> (HashMap<NodeId, Node>, Option<NodeId>) {
        (self.nodes.clone(), self.root_node_id)
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
        Self::add_direct(
            &mut self.nodes,
            self.id_generator
                .as_mut()
                .expect("non-authoritative graph tried to add node"),
            data,
        )
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    fn find_all_reachable_nodes(&self, node_id: NodeId, seen: &mut HashSet<NodeId>) {
        seen.insert(node_id);

        let node = self.get(node_id).unwrap();
        for child in node.children.iter().filter_map(|x| *x) {
            self.find_all_reachable_nodes(child, seen);
        }
    }

    fn garbage_collect(&mut self) {
        if let Some(root_node_id) = self.root_node_id {
            let all: HashSet<_> = self.nodes.keys().copied().collect();
            let mut seen = HashSet::new();
            self.find_all_reachable_nodes(root_node_id, &mut seen);

            for node_id in all.difference(&seen) {
                self.nodes.remove(node_id);
            }
        }
    }

    fn apply_event(&mut self, event: &GraphEvent) -> Option<()> {
        match event {
            GraphEvent::AddChild(parent_id, index, node_data) => {
                let (index, can_have_children) = {
                    let parent = self.get(*parent_id)?;
                    (
                        index.unwrap_or(parent.children.len()),
                        parent.data.can_have_children(),
                    )
                };
                if !can_have_children {
                    panic!("tried to add child to node without children");
                }

                let child_id = self.add(node_data.clone());
                let parent = self.get_mut(*parent_id)?;
                parent.add_child(index, child_id);
            }
            GraphEvent::RemoveChild(parent_id, child_id) => {
                let parent = self.get_mut(*parent_id)?;
                parent.remove_child(*child_id);
            }
            GraphEvent::AddNewParent(parent_id, child_id, node_data) => {
                let child_transform = self.get(*child_id)?.transform;

                assert!(node_data.can_have_children());
                let new_parent_id = self.add(node_data.clone());
                {
                    let parent = self.get_mut(new_parent_id)?;
                    parent.add_child(0, *child_id);
                    parent.transform = child_transform;
                }

                {
                    let child = self.get_mut(*child_id)?;
                    child.transform = Transform::new();
                }

                let parent = self.get_mut(*parent_id)?;
                parent.replace_child(*child_id, new_parent_id);
            }

            GraphEvent::ApplyDiff(node_id, diff) => {
                self.get_mut(*node_id)?.data.apply(diff.clone())
            }

            GraphEvent::SetColour(node_id, rgb) => {
                self.get_mut(*node_id)?.rgb = *rgb;
            }
            GraphEvent::SetTranslation(node_id, translation) => {
                self.get_mut(*node_id)?.transform.translation = *translation;
            }
            GraphEvent::SetRotation(node_id, rotation) => {
                self.get_mut(*node_id)?.transform.rotation = *rotation;
            }
            GraphEvent::SetScale(node_id, scale) => {
                self.get_mut(*node_id)?.transform.scale = *scale;
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

    pub fn root_node_id(&self) -> Option<NodeId> {
        self.root_node_id
    }
}
