use bevy::math::Vec3;
use shared::canonical as cn;

use crate::egui;
use egui_node_graph::{self as eng, NodeId};

pub mod sync;

pub struct NodeData {
    kind: NodeKind,
}
impl eng::NodeDataTrait for NodeData {
    type Response = Response;
    type UserState = GraphState;
    type DataType = DataType;
    type ValueType = Value;

    fn bottom_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: eng::NodeId,
        _graph: &Graph,
        _user_state: &Self::UserState,
    ) -> Vec<eng::NodeResponse<Response>>
    where
        Response: eng::UserResponseTrait,
    {
        vec![]
    }
}

#[derive(PartialEq, Eq)]
pub enum DataType {
    Scalar,
    Vec3,
    SignedDistance,
}
impl eng::DataTypeTrait for DataType {
    fn data_type_color(&self) -> egui::Color32 {
        match self {
            DataType::Scalar => egui::Color32::LIGHT_RED,
            DataType::Vec3 => egui::Color32::LIGHT_BLUE,
            DataType::SignedDistance => egui::Color32::LIGHT_GREEN,
        }
    }

    fn name(&self) -> &str {
        match self {
            DataType::Scalar => "scalar",
            DataType::Vec3 => "3d vector",
            DataType::SignedDistance => "signed distance",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Vec3 { value: Vec3 },
    Scalar { value: f32 },
    SignedDistance,
}
impl Value {
    pub fn try_to_vec3(self) -> anyhow::Result<Vec3> {
        match self {
            Self::Vec3 { value } => Ok(value),
            _ => anyhow::bail!("Invalid cast from {:?} to vec3", self),
        }
    }

    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        match self {
            Self::Scalar { value } => Ok(value),
            _ => anyhow::bail!("Invalid cast from {:?} to scalar", self),
        }
    }
}
impl eng::WidgetValueTrait for Value {
    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) {
        ui.label(param_name);
        match self {
            Value::Vec3 { value } => {
                ui.columns(3, |columns| {
                    {
                        let ui = &mut columns[0];
                        ui.label("x");
                        ui.add(egui::DragValue::new(&mut value.x));
                    }
                    {
                        let ui = &mut columns[1];
                        ui.label("y");
                        ui.add(egui::DragValue::new(&mut value.y));
                    }
                    {
                        let ui = &mut columns[2];
                        ui.label("z");
                        ui.add(egui::DragValue::new(&mut value.z));
                    }
                });
            }
            Value::Scalar { value } => {
                ui.add(egui::DragValue::new(value));
            }
            Value::SignedDistance => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {}
impl eng::UserResponseTrait for Response {}

#[derive(Default)]
pub struct GraphState {
    pub node_map: bimap::BiHashMap<cn::NodeId, eng::NodeId>,
    pub input_map: bimap::BiHashMap<cn::InputId, eng::InputId>,
    pub output_map: bimap::BiHashMap<cn::OutputId, eng::OutputId>,
    pub node_responses: Vec<eng::NodeResponse<Response>>,
}
impl GraphState {
    fn canonical_node_id_to_ui(&self, canonical_node_id: cn::NodeId) -> Option<eng::NodeId> {
        self.node_map.get_by_left(&canonical_node_id).copied()
    }

    fn ui_node_id_to_canonical(&self, ui_node_id: eng::NodeId) -> Option<cn::NodeId> {
        self.node_map.get_by_right(&ui_node_id).copied()
    }

    fn canonical_input_id_to_ui(&self, canonical_input_id: cn::InputId) -> Option<eng::InputId> {
        self.input_map.get_by_left(&canonical_input_id).copied()
    }

    fn ui_input_id_to_canonical(&self, ui_input_id: eng::InputId) -> Option<cn::InputId> {
        self.input_map.get_by_right(&ui_input_id).copied()
    }

    fn canonical_output_id_to_ui(
        &self,
        canonical_output_id: cn::OutputId,
    ) -> Option<eng::OutputId> {
        self.output_map.get_by_left(&canonical_output_id).copied()
    }

    fn ui_output_id_to_canonical(&self, ui_output_id: eng::OutputId) -> Option<cn::OutputId> {
        self.output_map.get_by_right(&ui_output_id).copied()
    }
}

#[derive(Clone, Copy)]
pub enum NodeKind {
    Sphere,
    Output,
    Union,
}
impl NodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            NodeKind::Sphere => "Sphere",
            NodeKind::Output => "Output",
            NodeKind::Union => "Union",
        }
    }
}
impl From<&cn::NodeData> for NodeKind {
    fn from(node: &cn::NodeData) -> Self {
        match node {
            cn::NodeData::Sphere { .. } => Self::Sphere,
            cn::NodeData::Output => Self::Output,
            cn::NodeData::Union => Self::Union,
        }
    }
}

#[derive(Clone)]
pub struct NodeTemplate {
    canonical_node: cn::NodeData,
}
impl NodeTemplate {
    fn new(canonical_node: cn::NodeData) -> NodeTemplate {
        NodeTemplate { canonical_node }
    }
}
impl eng::NodeTemplateTrait for NodeTemplate {
    type NodeData = NodeData;
    type DataType = DataType;
    type ValueType = Value;

    fn node_finder_label(&self) -> &str {
        NodeKind::from(&self.canonical_node).label()
    }

    fn node_graph_label(&self) -> String {
        self.node_finder_label().into()
    }

    fn user_data(&self) -> Self::NodeData {
        NodeData {
            kind: NodeKind::from(&self.canonical_node),
        }
    }

    fn build_node(&self, graph: &mut Graph, node_id: NodeId) {
        use eng::InputParamKind;

        let input_scalar = |graph: &mut Graph, name: &str, value: f32| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                DataType::Scalar,
                Value::Scalar { value },
                InputParamKind::ConstantOnly,
                true,
            );
        };

        let input_vec3 = |graph: &mut Graph, name: &str, value: Vec3| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                DataType::Vec3,
                Value::Vec3 { value },
                InputParamKind::ConstantOnly,
                true,
            );
        };

        let input_sd = |graph: &mut Graph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                DataType::SignedDistance,
                Value::SignedDistance,
                InputParamKind::ConnectionOnly,
                true,
            );
        };

        let output_sd = |graph: &mut Graph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), DataType::SignedDistance);
        };

        match self.canonical_node {
            cn::NodeData::Sphere { center, radius } => {
                input_vec3(graph, "center", center);
                input_scalar(graph, "radius", radius);
                output_sd(graph, "out");
            }
            cn::NodeData::Output => {
                input_sd(graph, "in");
            }
            cn::NodeData::Union => {
                input_sd(graph, "lhs");
                input_sd(graph, "rhs");
                output_sd(graph, "out");
            }
        }
    }
}

pub struct NodeTemplatesAll;
impl eng::NodeTemplateIter for NodeTemplatesAll {
    type Item = NodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            Self::Item::new(cn::NodeData::Sphere {
                center: Vec3::ZERO,
                radius: 1.0,
            }),
            Self::Item::new(cn::NodeData::Output),
            Self::Item::new(cn::NodeData::Union),
        ]
    }
}

pub type Graph = eng::Graph<NodeData, DataType, Value>;
pub type EditorState = eng::GraphEditorState<NodeData, DataType, Value, NodeTemplate, GraphState>;
