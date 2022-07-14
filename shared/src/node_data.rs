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

// TODO: consider using a macro to generate the NodeData enum members
// TODO: consider generating delta structs for each NodeData type using a macro
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeData {
    Sphere(Sphere),
    Cylinder(Cylinder),
    Torus(Torus),
    Plane(Plane),
    Capsule(Capsule),
    TaperedCapsule(TaperedCapsule),
    Cone(Cone),
    Box(Box),
    TorusSector(TorusSector),
    BiconvexLens(BiconvexLens),

    Union(Union),
    Intersect(Intersect),
    Subtract(Subtract),
}
impl NodeData {
    fn as_node_data_meta(&self) -> &dyn NodeDataMeta {
        match self {
            NodeData::Sphere(d) => d as &dyn NodeDataMeta,
            NodeData::Cylinder(d) => d as &dyn NodeDataMeta,
            NodeData::Torus(d) => d as &dyn NodeDataMeta,
            NodeData::Plane(d) => d as &dyn NodeDataMeta,
            NodeData::Capsule(d) => d as &dyn NodeDataMeta,
            NodeData::TaperedCapsule(d) => d as &dyn NodeDataMeta,
            NodeData::Cone(d) => d as &dyn NodeDataMeta,
            NodeData::Box(d) => d as &dyn NodeDataMeta,
            NodeData::TorusSector(d) => d as &dyn NodeDataMeta,
            NodeData::BiconvexLens(d) => d as &dyn NodeDataMeta,
            NodeData::Union(d) => d as &dyn NodeDataMeta,
            NodeData::Intersect(d) => d as &dyn NodeDataMeta,
            NodeData::Subtract(d) => d as &dyn NodeDataMeta,
        }
    }
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

pub const NODE_DATA_DEFAULTS: &[NodeData] = &[
    // primitives
    NodeData::Sphere(Sphere::new()),
    NodeData::Cylinder(Cylinder::new()),
    NodeData::Torus(Torus::new()),
    NodeData::Plane(Plane::new()),
    NodeData::Capsule(Capsule::new()),
    NodeData::TaperedCapsule(TaperedCapsule::new()),
    NodeData::Cone(Cone::new()),
    NodeData::Box(Box::new()),
    NodeData::TorusSector(TorusSector::new()),
    NodeData::BiconvexLens(BiconvexLens::new()),
    // operations
    NodeData::Union(Union::new()),
    NodeData::Intersect(Intersect::new()),
    NodeData::Subtract(Subtract::new()),
];
