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

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformDiff {
    pub translation: Option<Vec3>,
    pub rotation: Option<Quat>,
    pub scale: Option<f32>,
}
impl TransformDiff {
    pub fn into_option(self) -> Option<Self> {
        let has_changes =
            self.translation.is_some() || self.rotation.is_some() || self.scale.is_some();
        has_changes.then_some(self)
    }
}
impl Transform {
    pub fn apply(&mut self, diff: TransformDiff) {
        self.translation = diff.translation.unwrap_or(self.translation);
        self.rotation = diff.rotation.unwrap_or(self.rotation);
        self.scale = diff.scale.unwrap_or(self.scale);
    }
}
impl From<Transform> for TransformDiff {
    fn from(t: Transform) -> Self {
        TransformDiff {
            translation: Some(t.translation),
            rotation: Some(t.rotation),
            scale: Some(t.scale),
        }
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

    pub(crate) fn add_child(&mut self, index: usize, child_id: NodeId) -> NodeDiff {
        self.children
            .resize(self.children.len().max(index + 1), None);
        self.children[index] = Some(child_id);

        NodeDiff {
            children: Some(self.children.clone()),
            ..Default::default()
        }
    }

    pub(crate) fn remove_child(&mut self, to_remove_id: NodeId) -> NodeDiff {
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

        NodeDiff {
            children: Some(self.children.clone()),
            ..Default::default()
        }
    }

    pub(crate) fn replace_child(&mut self, old_child_id: NodeId, new_child_id: NodeId) -> NodeDiff {
        if let Some(child_slot) = self.children.iter_mut().find(|c| **c == Some(old_child_id)) {
            *child_slot = Some(new_child_id);
        };

        NodeDiff {
            children: Some(self.children.clone()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NodeDiff {
    pub rgb: Option<(f32, f32, f32)>,
    pub transform: Option<TransformDiff>,
    pub data: Option<NodeDataDiff>,
    pub children: Option<Vec<Option<NodeId>>>,
}
impl NodeDiff {
    pub fn into_option(self) -> Option<Self> {
        let has_changes = self.rgb.is_some()
            || self.transform.is_some()
            || self.data.is_some()
            || self.children.is_some();
        has_changes.then_some(self)
    }
}
impl Node {
    pub fn apply(&mut self, diff: NodeDiff) {
        if let Some(rgb) = diff.rgb {
            self.rgb = rgb;
        }
        if let Some(d) = diff.transform {
            self.transform.apply(d);
        }
        if let Some(d) = diff.data {
            self.data.apply(d);
        }
        if let Some(children) = diff.children {
            self.children = children;
        }
    }
}
