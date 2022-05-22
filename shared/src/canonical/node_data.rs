use super::DataType;

use bevy_math::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeData {
    Sphere { center: Vec3, radius: f32 },
    Output,
    Union,
}

impl NodeData {
    pub fn sphere(center: Vec3, radius: f32) -> NodeData {
        NodeData::Sphere { center, radius }
    }

    pub fn input_types(&self) -> &'static [DataType] {
        match self {
            NodeData::Sphere { .. } => &[],
            NodeData::Output => &[DataType::SignedDistance],
            NodeData::Union => &[DataType::SignedDistance, DataType::SignedDistance],
        }
    }

    pub fn output_types(&self) -> &'static [DataType] {
        match self {
            NodeData::Sphere { .. } => &[DataType::SignedDistance],
            NodeData::Output => &[],
            NodeData::Union => &[DataType::SignedDistance],
        }
    }
}
