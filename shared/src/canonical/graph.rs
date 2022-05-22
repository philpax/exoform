use super::{IdGenerator, InputId, NodeData, NodeId, OutputId};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataType {
    Scalar,
    Vec3,
    SignedDistance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Input(pub InputId, pub DataType);
use std::collections::{HashMap, HashSet};

use bevy_math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Output(pub OutputId, pub DataType);

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub data: NodeData,
    pub position: Vec2,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphEvent {
    AddNode(NodeId, NodeData, Vec2),
    Update(NodeId, NodeData, Vec2),
    Connect(OutputId, InputId),
    Disconnect(OutputId, InputId),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    edges: HashSet<(OutputId, InputId)>,
    incoming_events: Vec<GraphEvent>,
    outgoing_events: Vec<GraphEvent>,

    node_id_generator: IdGenerator<NodeId>,
    input_id_generator: IdGenerator<InputId>,
    output_id_generator: IdGenerator<OutputId>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: NodeData, position: Vec2) -> NodeId {
        let node_id = self.request_node_id();
        self.add_incoming_and_apply_event(GraphEvent::AddNode(node_id, node, position));
        node_id
    }

    pub fn request_node_id(&mut self) -> NodeId {
        self.node_id_generator.generate()
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn connect(&mut self, output_id: OutputId, input_id: InputId) {
        self.add_incoming_and_apply_event(GraphEvent::Connect(output_id, input_id))
    }

    pub fn connect_by_ids(&mut self, output: (NodeId, u8), input: (NodeId, u8)) {
        let output_id = self
            .get_node(output.0)
            .map(|n| n.outputs[output.1 as usize].0)
            .unwrap();

        let input_id = self
            .get_node(input.0)
            .map(|n| n.inputs[input.1 as usize].0)
            .unwrap();

        self.connect(output_id, input_id);
    }

    pub fn drain_arrived_incoming_events(&mut self) -> Vec<GraphEvent> {
        let incoming_events = self.incoming_events.clone();
        self.incoming_events.clear();
        incoming_events
    }

    fn apply_event(&mut self, event: &GraphEvent) {
        match event {
            GraphEvent::AddNode(node_id, node_data, position) => {
                let inputs = node_data
                    .input_types()
                    .iter()
                    .map(|dt| Input(self.input_id_generator.generate(), *dt))
                    .collect();

                let outputs = node_data
                    .output_types()
                    .iter()
                    .map(|dt| Output(self.output_id_generator.generate(), *dt))
                    .collect();

                self.nodes.insert(
                    *node_id,
                    Node {
                        id: *node_id,
                        data: node_data.clone(),
                        position: *position,
                        inputs,
                        outputs,
                    },
                );
            }
            GraphEvent::Update(node_id, node_data, position) => {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    node.data = node_data.clone();
                    node.position = *position;
                }
            }
            GraphEvent::Connect(output_id, input_id) => {
                self.edges.insert((*output_id, *input_id));
            }
            GraphEvent::Disconnect(output_id, input_id) => {
                self.edges.remove(&(*output_id, *input_id));
            }
        }
    }

    fn add_incoming_and_apply_event(&mut self, event: GraphEvent) {
        self.apply_event(&event);
        self.incoming_events.push(event);
    }

    pub fn add_outgoing_and_apply_event(&mut self, event: GraphEvent) {
        self.apply_event(&event);
        self.outgoing_events.push(event);
    }

    /// Get a reference to the graph's nodes.
    #[must_use]
    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }

    /// Get a reference to the graph's edges.
    #[must_use]
    pub fn edges(&self) -> &HashSet<(OutputId, InputId)> {
        &self.edges
    }

    /// Get a reference to the graph's incoming events.
    #[must_use]
    pub fn incoming_events(&self) -> &[GraphEvent] {
        self.incoming_events.as_ref()
    }
}
