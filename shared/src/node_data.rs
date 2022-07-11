use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::NodeCategory;

pub trait NodeDataMeta {
    fn name(&self) -> &'static str;
    fn category(&self) -> NodeCategory;

    fn can_have_children(&self) -> bool {
        false
    }
}

// Primitives

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sphere {
    pub radius: f32,
}
impl Sphere {
    pub const fn new() -> Sphere {
        Sphere { radius: 1.0 }
    }
}
impl Default for Sphere {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Sphere {
    fn name(&self) -> &'static str {
        "Sphere"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cylinder {
    pub cylinder_radius: f32,
    pub half_height: f32,
    pub rounding_radius: f32,
}
impl Cylinder {
    pub const fn new() -> Cylinder {
        Cylinder {
            cylinder_radius: 1.0,
            half_height: 1.0,
            rounding_radius: 0.0,
        }
    }
}
impl Default for Cylinder {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Cylinder {
    fn name(&self) -> &'static str {
        "Cylinder"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Torus {
    pub big_r: f32,
    pub small_r: f32,
}
impl Torus {
    pub const fn new() -> Torus {
        Torus {
            big_r: 1.0,
            small_r: 0.1,
        }
    }
}
impl Default for Torus {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Torus {
    fn name(&self) -> &'static str {
        "Torus"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    // *Must* be normalised!
    pub normal: Vec3,
    pub distance_from_origin: f32,
}
impl Plane {
    pub const fn new() -> Plane {
        let normal = glam::const_vec3!([0.0, 1.0, 0.0]);
        Plane {
            normal,
            distance_from_origin: 0.0,
        }
    }
}
impl Default for Plane {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Plane {
    fn name(&self) -> &'static str {
        "Plane"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capsule {
    pub points: [Vec3; 2],
    pub radius: f32,
}
impl Capsule {
    pub const fn new() -> Capsule {
        Capsule {
            points: [
                glam::const_vec3!([0.0, 0.0, 0.0]),
                glam::const_vec3!([0.0, 1.0, 0.0]),
            ],
            radius: 1.0,
        }
    }
}
impl Default for Capsule {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Capsule {
    fn name(&self) -> &'static str {
        "Capsule"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaperedCapsule {
    pub points: [Vec3; 2],
    pub radii: [f32; 2],
}
impl TaperedCapsule {
    pub const fn new() -> TaperedCapsule {
        TaperedCapsule {
            points: [
                glam::const_vec3!([0.0, 0.0, 0.0]),
                glam::const_vec3!([0.0, 1.0, 0.0]),
            ],
            radii: [1.0, 1.0],
        }
    }
}
impl Default for TaperedCapsule {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for TaperedCapsule {
    fn name(&self) -> &'static str {
        "Tapered Capsule"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cone {
    pub radius: f32,
    pub height: f32,
}
impl Cone {
    pub const fn new() -> Cone {
        Cone {
            radius: 1.0,
            height: 1.0,
        }
    }
}
impl Default for Cone {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Cone {
    fn name(&self) -> &'static str {
        "Cone"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Box {
    pub half_size: Vec3,
    pub rounding_radius: f32,
}
impl Box {
    pub const fn new() -> Box {
        Box {
            half_size: glam::const_vec3!([0.5, 0.5, 0.5]),
            rounding_radius: 0.0,
        }
    }
}
impl Default for Box {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Box {
    fn name(&self) -> &'static str {
        "Box"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TorusSector {
    pub big_r: f32,
    pub small_r: f32,
    pub angle: f32,
}
impl TorusSector {
    pub const fn new() -> TorusSector {
        TorusSector {
            big_r: 1.0,
            small_r: 0.1,
            angle: std::f32::consts::PI,
        }
    }
}
impl Default for TorusSector {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for TorusSector {
    fn name(&self) -> &'static str {
        "Torus Sector"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiconvexLens {
    pub lower_sagitta: f32,
    pub upper_sagitta: f32,
    pub chord: f32,
}
impl BiconvexLens {
    pub const fn new() -> BiconvexLens {
        BiconvexLens {
            lower_sagitta: 0.5,
            upper_sagitta: 0.5,
            chord: 0.5,
        }
    }
}
impl Default for BiconvexLens {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for BiconvexLens {
    fn name(&self) -> &'static str {
        "Biconvex Lens"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Primitive
    }
}

// Combinators

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Union {
    pub factor: f32,
}
impl Union {
    pub const fn new() -> Union {
        Union { factor: 0.0 }
    }
}
impl Default for Union {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Union {
    fn name(&self) -> &'static str {
        "Union"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Operation
    }
    fn can_have_children(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intersect {
    pub factor: f32,
}
impl Intersect {
    pub const fn new() -> Intersect {
        Intersect { factor: 0.0 }
    }
}
impl Default for Intersect {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Intersect {
    fn name(&self) -> &'static str {
        "Intersect"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Operation
    }
    fn can_have_children(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subtract {
    pub factor: f32,
}
impl Subtract {
    pub const fn new() -> Subtract {
        Subtract { factor: 0.0 }
    }
}
impl Default for Subtract {
    fn default() -> Self {
        Self::new()
    }
}
impl NodeDataMeta for Subtract {
    fn name(&self) -> &'static str {
        "Subtract"
    }
    fn category(&self) -> NodeCategory {
        NodeCategory::Operation
    }
    fn can_have_children(&self) -> bool {
        true
    }
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
