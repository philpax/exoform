use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::{NodeCategory, NodeId};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
    pub rgb: (f32, f32, f32),
    pub child: Option<NodeId>,
}
impl Rgb {
    pub const fn new() -> Rgb {
        Rgb {
            rgb: (1.0, 1.0, 1.0),
            child: None,
        }
    }
}
impl Default for Rgb {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Translate {
    pub position: Vec3,
    pub child: Option<NodeId>,
}
impl Translate {
    pub const fn new() -> Translate {
        Translate {
            position: Vec3::ZERO,
            child: None,
        }
    }
}
impl Default for Translate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rotate {
    pub rotation: Quat,
    pub child: Option<NodeId>,
}
impl Rotate {
    pub const fn new() -> Rotate {
        Rotate {
            rotation: Quat::IDENTITY,
            child: None,
        }
    }
}
impl Default for Rotate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub scale: f32,
    pub child: Option<NodeId>,
}
impl Scale {
    pub const fn new() -> Scale {
        Scale {
            scale: 1.0,
            child: None,
        }
    }
}
impl Default for Scale {
    fn default() -> Self {
        Self::new()
    }
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
    pub const fn name(&self) -> &str {
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

    pub const fn category(&self) -> NodeCategory {
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

        fn add_to_one(child: &mut Option<NodeId>, child_id: NodeId) {
            *child = Some(child_id)
        }

        match self {
            NodeData::Sphere(_) => panic!("this node does not support children"),
            NodeData::Cylinder(_) => panic!("this node does not support children"),
            NodeData::Torus(_) => panic!("this node does not support children"),

            NodeData::Union(Union { children, .. }) => add_to_vec(children, index, child_id),
            NodeData::Intersect(Intersect { children, .. }) => {
                add_to_lhs_rhs(children, index, child_id)
            }
            NodeData::Subtract(Subtract { children, .. }) => add_to_vec(children, index, child_id),

            NodeData::Rgb(Rgb { child, .. }) => add_to_one(child, child_id),

            NodeData::Translate(Translate { child, .. }) => add_to_one(child, child_id),
            NodeData::Rotate(Rotate { child, .. }) => add_to_one(child, child_id),
            NodeData::Scale(Scale { child, .. }) => add_to_one(child, child_id),
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

        fn remove_from_one(child: &mut Option<NodeId>) {
            *child = None
        }

        match self {
            NodeData::Sphere(_) => panic!("this node does not support children"),
            NodeData::Cylinder(_) => panic!("this node does not support children"),
            NodeData::Torus(_) => panic!("this node does not support children"),

            NodeData::Union(Union { children, .. }) => remove_from_vec(children, child_id),
            NodeData::Intersect(Intersect { children, .. }) => {
                remove_from_lhs_rhs(children, child_id)
            }
            NodeData::Subtract(Subtract { children, .. }) => remove_from_vec(children, child_id),

            NodeData::Rgb(Rgb { child, .. }) => remove_from_one(child),

            NodeData::Translate(Translate { child, .. }) => remove_from_one(child),
            NodeData::Rotate(Rotate { child, .. }) => remove_from_one(child),
            NodeData::Scale(Scale { child, .. }) => remove_from_one(child),
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

        fn replace_in_one(child: &mut Option<NodeId>, old_child_id: NodeId, new_child_id: NodeId) {
            if *child == Some(old_child_id) {
                *child = Some(new_child_id);
            }
        }

        match self {
            NodeData::Sphere(_) => panic!("this node does not support children"),
            NodeData::Cylinder(_) => panic!("this node does not support children"),
            NodeData::Torus(_) => panic!("this node does not support children"),

            NodeData::Union(Union { children, .. }) => {
                replace_in_vec(children, old_child_id, new_child_id)
            }
            NodeData::Intersect(Intersect { children, .. }) => {
                replace_in_lhs_rhs(children, old_child_id, new_child_id)
            }
            NodeData::Subtract(Subtract { children, .. }) => {
                replace_in_vec(children, old_child_id, new_child_id)
            }

            NodeData::Rgb(Rgb { child, .. }) => replace_in_one(child, old_child_id, new_child_id),

            NodeData::Translate(Translate { child, .. }) => {
                replace_in_one(child, old_child_id, new_child_id)
            }
            NodeData::Rotate(Rotate { child, .. }) => {
                replace_in_one(child, old_child_id, new_child_id)
            }
            NodeData::Scale(Scale { child, .. }) => {
                replace_in_one(child, old_child_id, new_child_id)
            }
        }
    }
}
