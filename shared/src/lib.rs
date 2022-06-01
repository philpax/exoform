use bevy_math::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeCategory {
    Primitive,
    Operation,
    Metadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Sphere {
        position: Vec3,
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
}
impl Node {
    pub fn name(&self) -> &str {
        match self {
            Node::Sphere { .. } => "Sphere",
            Node::Cylinder { .. } => "Cylinder",
            Node::Torus { .. } => "Torus",
            Node::Union(..) => "Union",
            Node::Intersect(..) => "Intersect",
            Node::Subtract(..) => "Subtract",
            Node::Rgb(..) => "Rgb",
        }
    }

    pub fn category(&self) -> NodeCategory {
        match self {
            Node::Sphere { .. } => NodeCategory::Primitive,
            Node::Cylinder { .. } => NodeCategory::Primitive,
            Node::Torus { .. } => NodeCategory::Primitive,
            Node::Union(..) => NodeCategory::Operation,
            Node::Intersect(..) => NodeCategory::Operation,
            Node::Subtract(..) => NodeCategory::Operation,
            Node::Rgb(..) => NodeCategory::Metadata,
        }
    }
}

pub const NODE_DEFAULTS: &[Node] = &[
    Node::Sphere {
        position: glam::Vec3::ZERO,
        radius: 0.0,
    },
    Node::Cylinder {
        cylinder_radius: 0.0,
        half_height: 0.0,
        rounding_radius: 0.0,
    },
    Node::Torus {
        big_r: 0.0,
        small_r: 0.0,
    },
    Node::Union(0.0, vec![]),
    Node::Intersect(0.0, (None, None)),
    Node::Subtract(0.0, (None, None)),
    Node::Rgb(1.0, 1.0, 1.0, None),
];

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
