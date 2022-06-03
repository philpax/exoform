use bevy_math::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeCategory {
    Primitive,
    Operation,
    Metadata,
    Transform,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeData {
    Sphere {
        radius: f32,
    },
    Cylinder {
        cylinder_radius: f32,
        half_height: f32,
        rounding_radius: f32,
    },
    Torus {
        big_r: f32,
        small_r: f32,
    },

    Union(f32, Vec<Node>),
    Intersect(f32, (Option<Box<Node>>, Option<Box<Node>>)),
    Subtract(f32, (Option<Box<Node>>, Option<Box<Node>>)),

    Rgb(f32, f32, f32, Option<Box<Node>>),

    Translate(Vec3, Option<Box<Node>>),
    Rotate(Quat, Option<Box<Node>>),
    Scale(f32, Option<Box<Node>>),
}
impl NodeData {
    pub fn name(&self) -> &str {
        match self {
            NodeData::Sphere { .. } => "Sphere",
            NodeData::Cylinder { .. } => "Cylinder",
            NodeData::Torus { .. } => "Torus",

            NodeData::Union(..) => "Union",
            NodeData::Intersect(..) => "Intersect",
            NodeData::Subtract(..) => "Subtract",

            NodeData::Rgb(..) => "Rgb",

            NodeData::Translate(..) => "Translate",
            NodeData::Rotate(..) => "Rotate",
            NodeData::Scale(..) => "Scale",
        }
    }

    pub fn category(&self) -> NodeCategory {
        match self {
            NodeData::Sphere { .. } => NodeCategory::Primitive,
            NodeData::Cylinder { .. } => NodeCategory::Primitive,
            NodeData::Torus { .. } => NodeCategory::Primitive,

            NodeData::Union(..) => NodeCategory::Operation,
            NodeData::Intersect(..) => NodeCategory::Operation,
            NodeData::Subtract(..) => NodeCategory::Operation,

            NodeData::Rgb(..) => NodeCategory::Metadata,

            NodeData::Translate(..) => NodeCategory::Transform,
            NodeData::Rotate(..) => NodeCategory::Transform,
            NodeData::Scale(..) => NodeCategory::Transform,
        }
    }
}

pub const NODE_DEFAULTS: &[NodeData] = &[
    NodeData::Sphere { radius: 1.0 },
    NodeData::Cylinder {
        cylinder_radius: 1.0,
        half_height: 1.0,
        rounding_radius: 0.0,
    },
    NodeData::Torus {
        big_r: 1.0,
        small_r: 0.1,
    },
    //
    NodeData::Union(0.0, vec![]),
    NodeData::Intersect(0.0, (None, None)),
    NodeData::Subtract(0.0, (None, None)),
    //
    NodeData::Rgb(1.0, 1.0, 1.0, None),
    //
    NodeData::Translate(Vec3::ZERO, None),
    NodeData::Rotate(Quat::IDENTITY, None),
    NodeData::Scale(1.0, None),
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: f32,
    pub data: NodeData,
}
impl Node {
    pub fn default_with_data(data: NodeData) -> Node {
        Node {
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
