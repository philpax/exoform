use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{node_data::*, NodeDiff, Transform, TransformDiff};
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

pub type GraphComponents = (HashMap<NodeId, Node>, Option<NodeId>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphCommand {
    AddChild(NodeId, Option<usize>, NodeData),
    RemoveChild(NodeId, NodeId),
    AddNewParent(NodeId, NodeId, NodeData),

    ApplyDiff(NodeId, NodeDiff),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphChange {
    Initialize(GraphComponents),
    CreateNode(NodeId, Node),
    DeleteNode(NodeId),
    ApplyDiff(NodeId, NodeDiff),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    root_node_id: Option<NodeId>,

    id_generator: Option<IdGenerator>,
}

impl Graph {
    pub fn new_authoritative(data: NodeData) -> Graph {
        let mut nodes = HashMap::new();
        let mut id_generator = IdGenerator::new();
        let root_node_id = Some({
            let id = id_generator.generate();
            nodes.insert(id, Node::new(id, data));
            id
        });
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

    fn from_components((nodes, root_node_id): GraphComponents) -> Graph {
        Graph {
            nodes,
            root_node_id,
            id_generator: None,
        }
    }

    pub fn to_components(&self) -> GraphComponents {
        (self.nodes.clone(), self.root_node_id)
    }

    fn is_authoritative(&self) -> bool {
        self.id_generator.is_some()
    }

    fn add(&mut self, data: NodeData, transform: Transform) -> (NodeId, GraphChange) {
        assert!(self.is_authoritative());
        let id = self.id_generator.as_mut().unwrap().generate();
        let node = Node {
            id,
            rgb: Node::DEFAULT_COLOUR,
            transform,
            data,
            children: vec![],
        };
        self.nodes.insert(id, node.clone());
        (id, GraphChange::CreateNode(id, node))
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

    fn garbage_collect(&mut self) -> Vec<GraphChange> {
        let root_node_id = match self.root_node_id {
            Some(it) => it,
            _ => return vec![],
        };
        let all: HashSet<_> = self.nodes.keys().copied().collect();
        let mut seen = HashSet::new();
        self.find_all_reachable_nodes(root_node_id, &mut seen);

        let ids: Vec<_> = all.difference(&seen).cloned().collect();
        for id in &ids {
            self.nodes.remove(id);
        }
        ids.into_iter().map(GraphChange::DeleteNode).collect()
    }

    fn apply_command(&mut self, command: &GraphCommand) -> Option<Vec<GraphChange>> {
        let mut changes = vec![];
        match command {
            GraphCommand::AddChild(parent_id, index, node_data) => {
                let parent_id = *parent_id;
                let (index, can_have_children) = {
                    let parent = self.get(parent_id)?;
                    (
                        index.unwrap_or(parent.children.len()),
                        parent.data.can_have_children(),
                    )
                };
                if !can_have_children {
                    panic!("tried to add child to node without children");
                }

                let (child_id, graph_change) = self.add(node_data.clone(), Transform::default());
                changes.push(graph_change);
                let add_child_diff = self.get_mut(parent_id)?.add_child(index, child_id);
                changes.push(GraphChange::ApplyDiff(parent_id, add_child_diff));
            }
            GraphCommand::RemoveChild(parent_id, child_id) => {
                let parent = self.get_mut(*parent_id)?;
                let remove_child_diff = parent.remove_child(*child_id);
                changes.push(GraphChange::ApplyDiff(*parent_id, remove_child_diff));
            }
            GraphCommand::AddNewParent(parent_id, child_id, node_data) => {
                assert!(node_data.can_have_children());

                let (new_parent_id, graph_change) = {
                    let child_transform = self.get(*child_id)?.transform;
                    self.add(node_data.clone(), child_transform)
                };
                changes.push(graph_change);

                let new_child_diff = self.get_mut(new_parent_id)?.add_child(0, *child_id);
                changes.push(GraphChange::ApplyDiff(new_parent_id, new_child_diff));

                {
                    let transform_diff = NodeDiff {
                        transform: Some(TransformDiff {
                            translation: Some(Default::default()),
                            rotation: Some(Default::default()),
                            scale: Some(Default::default()),
                        }),
                        ..Default::default()
                    };
                    self.get_mut(*child_id)?.apply(transform_diff.clone());
                    changes.push(GraphChange::ApplyDiff(*child_id, transform_diff));
                }

                let parent = self.get_mut(*parent_id)?;
                let replace_child_diff = parent.replace_child(*child_id, new_parent_id);
                changes.push(GraphChange::ApplyDiff(*parent_id, replace_child_diff));
            }

            GraphCommand::ApplyDiff(node_id, diff) => {
                self.get_mut(*node_id)?.apply(diff.clone());
                changes.push(GraphChange::ApplyDiff(*node_id, diff.clone()));
            }
        }
        Some(changes)
    }

    pub fn apply_commands(&mut self, commands: &[GraphCommand]) -> Vec<GraphChange> {
        assert!(self.is_authoritative());
        let mut ret = vec![];
        for command in commands {
            ret.append(
                &mut self
                    .apply_command(command)
                    .expect("failed to apply commands cleanly"),
            );
        }
        ret.append(&mut self.garbage_collect());
        ret
    }

    pub fn apply_changes(&mut self, changes: &[GraphChange]) {
        assert!(!self.is_authoritative());
        for change in changes {
            match change {
                GraphChange::Initialize(components) => {
                    *self = Self::from_components(components.clone());
                }
                GraphChange::CreateNode(node_id, node) => {
                    self.nodes.insert(*node_id, node.clone());
                }
                GraphChange::DeleteNode(node_id) => {
                    self.nodes.remove(node_id);
                }
                GraphChange::ApplyDiff(node_id, diff) => {
                    self.nodes
                        .get_mut(node_id)
                        .expect("failed to find node to apply change to")
                        .apply(diff.clone());
                }
            }
        }
    }

    pub fn root_node_id(&self) -> Option<NodeId> {
        self.root_node_id
    }
}
