use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{NodeCategory, NodeId};

pub enum ChildrenCount {
    None,
    Bounded(usize),
    Unbounded,
}

pub trait NodeDataMeta {
    fn name(&self) -> &'static str;
    fn category(&self) -> NodeCategory;

    fn children_count(&self) -> ChildrenCount {
        ChildrenCount::None
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
    pub children: Vec<NodeId>,
}
impl Union {
    pub const fn new() -> Union {
        Union {
            factor: 0.0,
            children: vec![],
        }
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
    fn children_count(&self) -> ChildrenCount {
        ChildrenCount::Unbounded
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intersect {
    pub factor: f32,
    pub children: (Option<NodeId>, Option<NodeId>),
}
impl Intersect {
    pub const fn new() -> Intersect {
        Intersect {
            factor: 0.0,
            children: (None, None),
        }
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
    fn children_count(&self) -> ChildrenCount {
        ChildrenCount::Bounded(2)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subtract {
    pub factor: f32,
    pub children: Vec<NodeId>,
}
impl Subtract {
    pub const fn new() -> Subtract {
        Subtract {
            factor: 0.0,
            children: vec![],
        }
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
    fn children_count(&self) -> ChildrenCount {
        ChildrenCount::Unbounded
    }
}

// TODO: consider using a macro to generate the NodeData enum members
// TODO: consider moving name/category to static methods on the structs, or using a trait
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
    pub const fn as_node_data_meta(&self) -> &dyn NodeDataMeta {
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

    pub fn name(&self) -> &str {
        self.as_node_data_meta().name()
    }

    pub fn category(&self) -> NodeCategory {
        self.as_node_data_meta().category()
    }

    pub(crate) fn add_child(&mut self, index: Option<usize>, child_id: NodeId) {
        fn add_to_vec(children: &mut Vec<NodeId>, index: Option<usize>, child_id: NodeId) {
            match index {
                Some(index) => *children.get_mut(index).unwrap() = child_id,
                None => children.push(child_id),
            }
        }

        fn add_to_lhs_rhs(
            children: &mut (Option<NodeId>, Option<NodeId>),
            index: Option<usize>,
            child_id: NodeId,
        ) {
            match index {
                Some(0) => children.0 = Some(child_id),
                Some(1) => children.1 = Some(child_id),
                Some(_) => panic!("out of bounds index"),
                None => match children {
                    (None, _) => children.0 = Some(child_id),
                    (_, None) => children.1 = Some(child_id),
                    (Some(_), Some(_)) => {
                        panic!("tried to add a new child, but both slots were full")
                    }
                },
            }
        }

        match self {
            NodeData::Sphere(_) => panic!("this node does not support children"),
            NodeData::Cylinder(_) => panic!("this node does not support children"),
            NodeData::Torus(_) => panic!("this node does not support children"),
            NodeData::Plane(_) => panic!("this node does not support children"),
            NodeData::Capsule(_) => panic!("this node does not support children"),
            NodeData::TaperedCapsule(_) => panic!("this node does not support children"),
            NodeData::Cone(_) => panic!("this node does not support children"),
            NodeData::Box(_) => panic!("this node does not support children"),
            NodeData::TorusSector(_) => panic!("this node does not support children"),
            NodeData::BiconvexLens(_) => panic!("this node does not support children"),

            NodeData::Union(Union { children, .. }) => add_to_vec(children, index, child_id),
            NodeData::Intersect(Intersect { children, .. }) => {
                add_to_lhs_rhs(children, index, child_id)
            }
            NodeData::Subtract(Subtract { children, .. }) => add_to_vec(children, index, child_id),
        }
    }

    pub(crate) fn remove_child(&mut self, child_id: NodeId) {
        fn remove_from_vec(children: &mut Vec<NodeId>, child_id: NodeId) {
            children.retain(|id| *id != child_id);
        }

        fn remove_from_lhs_rhs(children: &mut (Option<NodeId>, Option<NodeId>), child_id: NodeId) {
            if children.0 == Some(child_id) {
                children.0 = None;
            }

            if children.1 == Some(child_id) {
                children.1 = None;
            }
        }

        match self {
            NodeData::Sphere(_) => panic!("this node does not support children"),
            NodeData::Cylinder(_) => panic!("this node does not support children"),
            NodeData::Torus(_) => panic!("this node does not support children"),
            NodeData::Plane(_) => panic!("this node does not support children"),
            NodeData::Capsule(_) => panic!("this node does not support children"),
            NodeData::TaperedCapsule(_) => panic!("this node does not support children"),
            NodeData::Cone(_) => panic!("this node does not support children"),
            NodeData::Box(_) => panic!("this node does not support children"),
            NodeData::TorusSector(_) => panic!("this node does not support children"),
            NodeData::BiconvexLens(_) => panic!("this node does not support children"),

            NodeData::Union(Union { children, .. }) => remove_from_vec(children, child_id),
            NodeData::Intersect(Intersect { children, .. }) => {
                remove_from_lhs_rhs(children, child_id)
            }
            NodeData::Subtract(Subtract { children, .. }) => remove_from_vec(children, child_id),
        }
    }

    pub(crate) fn replace_child(&mut self, old_child_id: NodeId, new_child_id: NodeId) {
        fn replace_in_vec(children: &mut Vec<NodeId>, old_child_id: NodeId, new_child_id: NodeId) {
            for child_id in children {
                if *child_id == old_child_id {
                    *child_id = new_child_id;
                }
            }
        }

        fn replace_in_lhs_rhs(
            children: &mut (Option<NodeId>, Option<NodeId>),
            old_child_id: NodeId,
            new_child_id: NodeId,
        ) {
            if children.0 == Some(old_child_id) {
                children.0 = Some(new_child_id);
            }

            if children.1 == Some(old_child_id) {
                children.1 = Some(new_child_id);
            }
        }

        match self {
            NodeData::Sphere(_) => panic!("this node does not support children"),
            NodeData::Cylinder(_) => panic!("this node does not support children"),
            NodeData::Torus(_) => panic!("this node does not support children"),
            NodeData::Plane(_) => panic!("this node does not support children"),
            NodeData::Capsule(_) => panic!("this node does not support children"),
            NodeData::TaperedCapsule(_) => panic!("this node does not support children"),
            NodeData::Cone(_) => panic!("this node does not support children"),
            NodeData::Box(_) => panic!("this node does not support children"),
            NodeData::TorusSector(_) => panic!("this node does not support children"),
            NodeData::BiconvexLens(_) => panic!("this node does not support children"),

            NodeData::Union(Union { children, .. }) => {
                replace_in_vec(children, old_child_id, new_child_id)
            }
            NodeData::Intersect(Intersect { children, .. }) => {
                replace_in_lhs_rhs(children, old_child_id, new_child_id)
            }
            NodeData::Subtract(Subtract { children, .. }) => {
                replace_in_vec(children, old_child_id, new_child_id)
            }
        }
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
