use derive_macros::node_type;
use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::NodeCategory;

pub trait NodeDataMeta {
    fn name(&self) -> &'static str;
    fn category(&self) -> NodeCategory;
    fn can_have_children(&self) -> bool;
}

// Primitives

#[node_type(name = "Sphere", category = NodeCategory::Primitive)]
pub struct Sphere {
    #[field(name = "Radius", default = 1.0)]
    radius: f32,
}

#[node_type(name = "Cylinder", category = NodeCategory::Primitive)]
pub struct Cylinder {
    #[field(name = "Cylinder radius", default = 1.0)]
    cylinder_radius: f32,
    #[field(name = "Half-height", default = 1.0)]
    half_height: f32,
    #[field(name = "Rounding radius", default = 0.0)]
    rounding_radius: f32,
}

#[node_type(name = "Torus", category = NodeCategory::Primitive)]
pub struct Torus {
    #[field(name = "Big radius", default = 1.0)]
    big_r: f32,
    #[field(name = "Small radius", default = 0.1)]
    small_r: f32,
}

#[node_type(name = "Plane", category = NodeCategory::Primitive)]
pub struct Plane {
    // *Must* be normalised!
    #[field(name = "Normal", default = glam::const_vec3!([0.0, 1.0, 0.0]))]
    pub normal: Vec3,
    #[field(name = "Distance from origin", default = 0.0)]
    pub distance_from_origin: f32,
}

#[node_type(name = "Capsule", category = NodeCategory::Primitive)]
pub struct Capsule {
    #[field(name = "Points", default = [
        glam::const_vec3!([0.0, 0.0, 0.0]),
        glam::const_vec3!([0.0, 1.0, 0.0]),
    ])]
    pub points: [Vec3; 2],
    #[field(name = "Radius", default = 1.0)]
    pub radius: f32,
}

#[node_type(name = "Tapered Capsule", category = NodeCategory::Primitive)]
pub struct TaperedCapsule {
    #[field(name = "Points", default = [
        glam::const_vec3!([0.0, 0.0, 0.0]),
        glam::const_vec3!([0.0, 1.0, 0.0]),
    ])]
    pub points: [Vec3; 2],
    #[field(name = "Radius", default = [1.0, 1.0])]
    pub radii: [f32; 2],
}

#[node_type(name = "Cone", category = NodeCategory::Primitive)]
pub struct Cone {
    #[field(name = "Radius", default = 1.0)]
    pub radius: f32,
    #[field(name = "Height", default = 1.0)]
    pub height: f32,
}

#[node_type(name = "Box", category = NodeCategory::Primitive)]
pub struct Box {
    #[field(name = "Half-size", default = glam::const_vec3!([0.5, 0.5, 0.5]))]
    pub half_size: Vec3,
    #[field(name = "Rounding radius", default = 0.0)]
    pub rounding_radius: f32,
}

#[node_type(name = "Torus Sector", category = NodeCategory::Primitive)]
pub struct TorusSector {
    #[field(name = "Big radius", default = 1.0)]
    pub big_r: f32,
    #[field(name = "Small radius", default = 0.1)]
    pub small_r: f32,
    #[field(name = "Angle", default = std::f32::consts::PI)]
    pub angle: f32,
}

#[node_type(name = "Biconvex Lens", category = NodeCategory::Primitive)]
pub struct BiconvexLens {
    #[field(name = "Lower sagitta", default = 0.5)]
    pub lower_sagitta: f32,
    #[field(name = "Upper sagitta", default = 0.5)]
    pub upper_sagitta: f32,
    #[field(name = "Chord", default = 0.5)]
    pub chord: f32,
}

// Operations

#[node_type(name = "Union", category = NodeCategory::Operation, children = true)]
pub struct Union {
    #[field(name = "Factor", default = 0.0)]
    pub factor: f32,
}

#[node_type(name = "Intersect", category = NodeCategory::Operation, children = true)]
pub struct Intersect {
    #[field(name = "Factor", default = 0.0)]
    pub factor: f32,
}

#[node_type(name = "Subtract", category = NodeCategory::Operation, children = true)]
pub struct Subtract {
    #[field(name = "Factor", default = 0.0)]
    pub factor: f32,
}

// TODO: consider generating delta structs for each NodeData type using a macro

macro_rules! generate_node_data {
    ($($i:ident),*) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum NodeData {
            $($i($i)),*
        }
        impl NodeDataMeta for NodeData {
            fn name(&self) -> &'static str {
                self.as_node_data_meta().name()
            }
            fn category(&self) -> NodeCategory {
                self.as_node_data_meta().category()
            }
            fn can_have_children(&self) -> bool {
                self.as_node_data_meta().can_have_children()
            }
        }
        impl NodeData {
            fn as_node_data_meta(&self) -> &dyn NodeDataMeta {
                match self {
                    $(NodeData::$i(d) => d as &dyn NodeDataMeta),*
                }
            }
        }
        pub const NODE_DATA_DEFAULTS: &[NodeData] = &[
            $(NodeData::$i(self::$i::new())),*
        ];
    }
}

generate_node_data!(
    Sphere,
    Cylinder,
    Torus,
    Plane,
    Capsule,
    TaperedCapsule,
    Cone,
    Box,
    TorusSector,
    BiconvexLens,
    Union,
    Intersect,
    Subtract
);
