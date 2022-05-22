use crate::ui::graph::{self as uig, NodeKind, Value};
use egui_node_graph::NodeTemplateTrait;
use shared::canonical as cn;

use anyhow::Context;
use bevy::math::Vec2;

pub fn link_canonical_and_ui_io(
    graph: &cn::Graph,
    sdf_editor_state: &mut uig::EditorState,
    canonical_node_id: cn::NodeId,
    ui_node_id: uig::NodeId,
) {
    let canonical_node = graph.get_node(canonical_node_id).unwrap();
    let ui_node = sdf_editor_state.graph.nodes.get(ui_node_id).unwrap();
    let ui_node_inputs = ui_node.inputs(&sdf_editor_state.graph).map(|i| i.id);
    let ui_node_outputs = ui_node.outputs(&sdf_editor_state.graph).map(|i| i.id);

    sdf_editor_state.user_state.input_map.extend(
        Iterator::zip(ui_node_inputs, canonical_node.inputs.iter())
            .map(|(editor_id, canonical)| (canonical.0, editor_id)),
    );
    sdf_editor_state.user_state.output_map.extend(
        Iterator::zip(ui_node_outputs, canonical_node.outputs.iter())
            .map(|(editor_id, canonical)| (canonical.0, editor_id)),
    );
}

pub fn sync_ui_to_canonical(
    response: &egui_node_graph::NodeResponse<uig::Response>,
    sdf_editor_state: &mut uig::EditorState,
    graph: &mut cn::Graph,
) {
    use egui_node_graph::NodeResponse;

    let event = match response {
        NodeResponse::ConnectEventEnded { output, input } => {
            let us = &sdf_editor_state.user_state;
            Some(cn::GraphEvent::Connect(
                us.ui_output_id_to_canonical(*output).unwrap(),
                us.ui_input_id_to_canonical(*input).unwrap(),
            ))
        }
        NodeResponse::CreatedNode(ui_node_id) => {
            // Generate a new node ID and use it to link the UI and canonical representations together.
            let node_id = graph.request_node_id();
            sdf_editor_state
                .user_state
                .node_map
                .insert(node_id, *ui_node_id);

            // Issue our spawn request.
            let (_, node_data, position) =
                extract_canonical_data_from_ui_node(sdf_editor_state, *ui_node_id).unwrap();
            Some(cn::GraphEvent::AddNode(node_id, node_data, position))
        }
        NodeResponse::DeleteNode(_) => todo!(),
        NodeResponse::DisconnectEvent { output, input } => {
            let us = &sdf_editor_state.user_state;
            Some(cn::GraphEvent::Disconnect(
                us.ui_output_id_to_canonical(*output).unwrap(),
                us.ui_input_id_to_canonical(*input).unwrap(),
            ))
        }
        NodeResponse::RaiseNode(ui_node_id) => {
            let (node_id, node_data, position) =
                extract_canonical_data_from_ui_node(sdf_editor_state, *ui_node_id).unwrap();

            Some(cn::GraphEvent::Update(node_id, node_data, position))
        }
        _ => None,
    };

    if let Some(event) = event {
        graph.add_outgoing_and_apply_event(event);
    }
}

fn extract_canonical_data_from_ui_node(
    sdf_editor_state: &uig::EditorState,
    ui_node_id: uig::NodeId,
) -> anyhow::Result<(cn::NodeId, cn::NodeData, Vec2)> {
    let ui_node = sdf_editor_state
        .graph
        .nodes
        .get(ui_node_id)
        .context("failed to get ui node")?;

    let canonical_id = sdf_editor_state
        .user_state
        .ui_node_id_to_canonical(ui_node_id)
        .context("failed to get canonical id")?;
    let node_data = ui_node_to_canonical_data(ui_node, &sdf_editor_state.graph)?;
    let position = get_ui_node_position(sdf_editor_state, ui_node_id)
        .context("failed to get ui node position")?;
    Ok((canonical_id, node_data, position))
}

fn get_ui_node_position(
    sdf_editor_state: &uig::EditorState,
    ui_node_id: uig::NodeId,
) -> Option<Vec2> {
    let position: [f32; 2] = sdf_editor_state.node_positions.get(ui_node_id)?.into();

    Some(position.into())
}

fn ui_node_to_canonical_data(
    ui_node: &egui_node_graph::Node<uig::NodeData>,
    graph: &uig::Graph,
) -> anyhow::Result<cn::NodeData> {
    let inputs: Vec<_> = ui_node.inputs(graph).map(|i| i.value).collect();

    Ok(match ui_node.user_data.kind {
        NodeKind::Sphere => cn::NodeData::Sphere {
            center: inputs[0].try_to_vec3()?,
            radius: inputs[1].try_to_scalar()?,
        },
        NodeKind::Output => cn::NodeData::Output,
        NodeKind::Union => cn::NodeData::Union,
    })
}

fn canonical_data_to_ui_node(node_data: &cn::NodeData) -> Vec<Value> {
    match node_data {
        cn::NodeData::Sphere { center, radius } => vec![
            Value::Vec3 { value: *center },
            Value::Scalar { value: *radius },
        ],
        cn::NodeData::Output => vec![],
        cn::NodeData::Union => vec![],
    }
}

pub fn sync_canonical_to_ui(
    event: cn::GraphEvent,
    sdf_editor_state: &mut uig::EditorState,
    graph: &cn::Graph,
) {
    match event {
        cn::GraphEvent::AddNode(node_id, node_data, position) => {
            // Add the node to the graph.
            let node_template = uig::NodeTemplate {
                canonical_node: node_data,
            };
            let ui_node_id = sdf_editor_state.graph.add_node(
                node_template.node_graph_label(),
                node_template.user_data(),
                |graph, node_id| node_template.build_node(graph, node_id),
            );
            sdf_editor_state
                .node_positions
                .insert(ui_node_id, position.to_array().into());
            sdf_editor_state.node_order.push(ui_node_id);

            // Link our canonical and UI nodes together.
            let canonical_node = graph.get_node(node_id).unwrap();
            sdf_editor_state
                .user_state
                .node_map
                .insert(canonical_node.id, ui_node_id);

            link_canonical_and_ui_io(graph, sdf_editor_state, node_id, ui_node_id);
        }
        cn::GraphEvent::Update(node_id, node_data, position) => {
            // Grab our UI node.
            let ui_node_id = sdf_editor_state
                .user_state
                .canonical_node_id_to_ui(node_id)
                .unwrap();
            let ui_node = sdf_editor_state.graph.nodes.get(ui_node_id).unwrap();

            // Update it.
            let new_inputs = canonical_data_to_ui_node(&node_data);
            for (input_id, new_value) in ui_node.input_ids().zip(new_inputs.into_iter()) {
                sdf_editor_state.graph.inputs[input_id].value = new_value;
            }
            sdf_editor_state.node_positions[ui_node_id] = position.to_array().into();
        }
        cn::GraphEvent::Connect(output_id, input_id) => {
            let us = &sdf_editor_state.user_state;
            sdf_editor_state.graph.add_connection(
                us.canonical_output_id_to_ui(output_id).unwrap(),
                us.canonical_input_id_to_ui(input_id).unwrap(),
            );
        }
        cn::GraphEvent::Disconnect(output_id, input_id) => {
            let us = &sdf_editor_state.user_state;
            sdf_editor_state.graph.remove_connection(
                us.canonical_input_id_to_ui(input_id).unwrap(),
                us.canonical_output_id_to_ui(output_id).unwrap(),
            );
        }
    }
}
