use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::node_data::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub(crate) u32);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeCategory {
    Primitive,
    Operation,
    Metadata,
    Transform,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: f32,
}
impl Transform {
    pub const fn new() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
        }
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub rgb: (f32, f32, f32),
    pub transform: Transform,
    pub data: NodeData,
    pub children: Vec<Option<NodeId>>,
}
impl Node {
    pub const DEFAULT_COLOUR: (f32, f32, f32) = (1.0, 1.0, 1.0);

    pub const fn new(id: NodeId, data: NodeData) -> Node {
        Node {
            id,
            rgb: Self::DEFAULT_COLOUR,
            transform: Transform::new(),
            data,
            children: vec![],
        }
    }

    pub(crate) fn add_child(&mut self, index: usize, child_id: NodeId) {
        self.children
            .resize(self.children.len().max(index + 1), None);
        self.children[index] = Some(child_id);
    }

    pub(crate) fn remove_child(&mut self, to_remove_id: NodeId) {
        for child_id in &mut self.children {
            if *child_id == Some(to_remove_id) {
                *child_id = None;
            }
        }
        if let Some((last_some_idx, _)) = self
            .children
            .iter()
            .enumerate()
            .rfind(|(_, val)| val.is_some())
        {
            self.children.truncate(last_some_idx + 1);
        }

        if self.children.iter().all(Option::is_none) {
            self.children.clear();
        }
    }

    pub(crate) fn replace_child(&mut self, old_child_id: NodeId, new_child_id: NodeId) {
        if let Some(child_slot) = self.children.iter_mut().find(|c| **c == Some(old_child_id)) {
            *child_slot = Some(new_child_id);
        };
    }
}
impl ToString for Node {
    fn to_string(&self) -> String {
        let mut buf = Vec::new();
        let mut serializer = serde_json::ser::Serializer::with_formatter(
            &mut buf,
            serde_json::ser::PrettyFormatter::with_indent(b" "),
        );
        self.serialize(&mut serializer).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
