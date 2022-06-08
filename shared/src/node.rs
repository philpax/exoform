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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub rgb: (f32, f32, f32),
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: f32,
    pub data: NodeData,
}
impl Node {
    pub const DEFAULT_COLOUR: (f32, f32, f32) = (1.0, 1.0, 1.0);

    pub const fn new(id: NodeId, data: NodeData) -> Node {
        Node {
            id,
            rgb: Self::DEFAULT_COLOUR,
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
            data,
        }
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

pub const NODE_DEFAULTS: &[NodeData] = &[
    NodeData::Sphere(Sphere::new()),
    NodeData::Cylinder(Cylinder::new()),
    NodeData::Torus(Torus::new()),
    NodeData::Union(Union::new()),
    NodeData::Intersect(Intersect::new()),
    NodeData::Subtract(Subtract::new()),
];
