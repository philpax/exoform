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
pub struct Sphere {
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cylinder {
    pub cylinder_radius: f32,
    pub half_height: f32,
    pub rounding_radius: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Torus {
    pub big_r: f32,
    pub small_r: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Union {
    pub factor: f32,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intersect {
    pub factor: f32,
    pub children: (Option<Box<Node>>, Option<Box<Node>>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subtract {
    pub factor: f32,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub child: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Translate {
    pub position: Vec3,
    pub child: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rotate {
    pub rotation: Quat,
    pub child: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub scale: f32,
    pub child: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeData {
    Sphere(Sphere),
    Cylinder(Cylinder),
    Torus(Torus),

    Union(Union),
    Intersect(Intersect),
    Subtract(Subtract),

    Rgb(Rgb),

    Translate(Translate),
    Rotate(Rotate),
    Scale(Scale),
}
impl NodeData {
    pub fn name(&self) -> &str {
        match self {
            NodeData::Sphere(_) => "Sphere",
            NodeData::Cylinder(_) => "Cylinder",
            NodeData::Torus(_) => "Torus",

            NodeData::Union(_) => "Union",
            NodeData::Intersect(_) => "Intersect",
            NodeData::Subtract(_) => "Subtract",

            NodeData::Rgb(_) => "Rgb",

            NodeData::Translate(_) => "Translate",
            NodeData::Rotate(_) => "Rotate",
            NodeData::Scale(_) => "Scale",
        }
    }

    pub fn category(&self) -> NodeCategory {
        match self {
            NodeData::Sphere(_) => NodeCategory::Primitive,
            NodeData::Cylinder(_) => NodeCategory::Primitive,
            NodeData::Torus(_) => NodeCategory::Primitive,

            NodeData::Union(_) => NodeCategory::Operation,
            NodeData::Intersect(_) => NodeCategory::Operation,
            NodeData::Subtract(_) => NodeCategory::Operation,

            NodeData::Rgb(_) => NodeCategory::Metadata,

            NodeData::Translate(_) => NodeCategory::Transform,
            NodeData::Rotate(_) => NodeCategory::Transform,
            NodeData::Scale(_) => NodeCategory::Transform,
        }
    }
}

pub const NODE_DEFAULTS: &[NodeData] = &[
    NodeData::Sphere(Sphere { radius: 1.0 }),
    NodeData::Cylinder(Cylinder {
        cylinder_radius: 1.0,
        half_height: 1.0,
        rounding_radius: 0.0,
    }),
    NodeData::Torus(Torus {
        big_r: 1.0,
        small_r: 0.1,
    }),
    //
    NodeData::Union(Union {
        factor: 0.0,
        children: vec![],
    }),
    NodeData::Intersect(Intersect {
        factor: 0.0,
        children: (None, None),
    }),
    NodeData::Subtract(Subtract {
        factor: 0.0,
        children: vec![],
    }),
    //
    NodeData::Rgb(Rgb {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        child: None,
    }),
    //
    NodeData::Translate(Translate {
        position: Vec3::ZERO,
        child: None,
    }),
    NodeData::Rotate(Rotate {
        rotation: Quat::IDENTITY,
        child: None,
    }),
    NodeData::Scale(Scale {
        scale: 1.0,
        child: None,
    }),
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
