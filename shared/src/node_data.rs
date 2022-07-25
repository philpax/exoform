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
    #[field(name = "Radius", default = 0.5)]
    radius: f32,
}

#[node_type(name = "Cylinder", category = NodeCategory::Primitive)]
pub struct Cylinder {
    #[field(name = "Cylinder radius", default = 0.5)]
    cylinder_radius: f32,
    #[field(name = "Half-height", default = 0.5)]
    half_height: f32,
    #[field(name = "Rounding radius", default = 0.0)]
    rounding_radius: f32,
}

#[node_type(name = "Torus", category = NodeCategory::Primitive)]
pub struct Torus {
    #[field(name = "Big radius", default = 0.5)]
    big_r: f32,
    #[field(name = "Small radius", default = 0.1)]
    small_r: f32,
}

#[node_type(name = "Plane", category = NodeCategory::Primitive)]
pub struct Plane {
    // *Must* be normalised!
    #[field(name = "Normal", default = glam::const_vec3!([0.0, 1.0, 0.0]))]
    normal: Vec3,
    #[field(name = "Distance from origin", default = 0.0)]
    distance_from_origin: f32,
}

#[node_type(name = "Capsule", category = NodeCategory::Primitive)]
pub struct Capsule {
    #[field(name = "Point 1", default = glam::const_vec3!([0.0, -0.5, 0.0]))]
    point_1: Vec3,
    #[field(name = "Point 2", default = glam::const_vec3!([0.0, 0.5, 0.0]))]
    point_2: Vec3,
    #[field(name = "Radius", default = 0.5)]
    radius: f32,
}

#[node_type(name = "Tapered Capsule", category = NodeCategory::Primitive)]
pub struct TaperedCapsule {
    #[field(name = "Point 1", default = glam::const_vec3!([0.0, -0.5, 0.0]))]
    point_1: Vec3,
    #[field(name = "Point 2", default = glam::const_vec3!([0.0, 0.5, 0.0]))]
    point_2: Vec3,
    #[field(name = "Radius 1", default = 0.5)]
    radius_1: f32,
    #[field(name = "Radius 2", default = 0.5)]
    radius_2: f32,
}

#[node_type(name = "Cone", category = NodeCategory::Primitive)]
pub struct Cone {
    #[field(name = "Radius", default = 0.5)]
    radius: f32,
    #[field(name = "Height", default = 1.0)]
    height: f32,
}

#[node_type(name = "Box", category = NodeCategory::Primitive)]
pub struct Box {
    #[field(name = "Half-size", default = glam::const_vec3!([0.5, 0.5, 0.5]))]
    half_size: Vec3,
    #[field(name = "Rounding radius", default = 0.0)]
    rounding_radius: f32,
}

#[node_type(name = "Torus Sector", category = NodeCategory::Primitive)]
pub struct TorusSector {
    #[field(name = "Big radius", default = 0.5)]
    big_r: f32,
    #[field(name = "Small radius", default = 0.1)]
    small_r: f32,
    #[field(name = "Angle", default = std::f32::consts::PI)]
    angle: f32,
}

#[node_type(name = "Biconvex Lens", category = NodeCategory::Primitive)]
pub struct BiconvexLens {
    #[field(name = "Lower sagitta", default = 0.5)]
    lower_sagitta: f32,
    #[field(name = "Upper sagitta", default = 0.5)]
    upper_sagitta: f32,
    #[field(name = "Chord", default = 1.0)]
    chord: f32,
}

// Operations

#[node_type(name = "Union", category = NodeCategory::Operation, children = true)]
pub struct Union {
    #[field(name = "Factor", default = 0.0)]
    factor: f32,
}

#[node_type(name = "Intersect", category = NodeCategory::Operation, children = true)]
pub struct Intersect {
    #[field(name = "Factor", default = 0.0)]
    factor: f32,
}

#[node_type(name = "Subtract", category = NodeCategory::Operation, children = true)]
pub struct Subtract {
    #[field(name = "Factor", default = 0.0)]
    factor: f32,
}

macro_rules! generate_node_data {
    ($(($ty:ident, $diff:ident)),*) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum NodeData {
            $($ty($ty)),*
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
                    $(NodeData::$ty(d) => d as &dyn NodeDataMeta),*
                }
            }
            pub fn apply(&mut self, diff: NodeDataDiff) {
                match (self, diff) {
                    $((NodeData::$ty(i), NodeDataDiff::$diff(d)) => i.apply(d)),*,
                    _ => {}
                }
            }
        }
        $(impl From<$ty> for NodeData {
            fn from(data: $ty) -> NodeData {
                NodeData::$ty(data)
            }
        })*
        pub const NODE_DATA_DEFAULTS: &[NodeData] = &[
            $(NodeData::$ty(self::$ty::new())),*
        ];
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum NodeDataDiff {
            $($diff($diff)),*
        }
        $(impl From<$diff> for NodeDataDiff {
            fn from(diff: $diff) -> NodeDataDiff {
                NodeDataDiff::$diff(diff)
            }
        })*
    }
}

generate_node_data!(
    (Sphere, SphereDiff),
    (Cylinder, CylinderDiff),
    (Torus, TorusDiff),
    (Plane, PlaneDiff),
    (Capsule, CapsuleDiff),
    (TaperedCapsule, TaperedCapsuleDiff),
    (Cone, ConeDiff),
    (Box, BoxDiff),
    (TorusSector, TorusSectorDiff),
    (BiconvexLens, BiconvexLensDiff),
    (Union, UnionDiff),
    (Intersect, IntersectDiff),
    (Subtract, SubtractDiff)
);
