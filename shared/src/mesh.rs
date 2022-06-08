use glam::{Quat, Vec3};

use crate::node_data::*;
use crate::{Graph, NodeId};

#[derive(Clone, Default)]
pub struct Mesh {
    pub indices: Vec<u32>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
}

pub fn generate_mesh(graph: &Graph) -> Option<Mesh> {
    let mut saft_graph = saft::Graph::default();
    let root_id = node_to_saft_node(&mut saft_graph, graph, graph.root_node_id)?;

    if saft_graph.bounding_box(root_id).volume() == 0.0 {
        return None;
    }
    let mesh = saft::mesh_from_sdf(&saft_graph, root_id, saft::MeshOptions::default()).ok()?;
    Some(Mesh {
        indices: mesh.indices,
        positions: mesh.positions,
        normals: mesh.normals,
        colors: mesh.colors,
    })
}

fn node_to_saft_node(
    saft_graph: &mut saft::Graph,
    graph: &Graph,
    node: NodeId,
) -> Option<saft::NodeId> {
    let node = graph.get(node).unwrap();
    let mut node_id = node_to_saft_node_data(saft_graph, graph, &node.data)?;
    if node.scale != 1.0 {
        node_id = saft_graph.op_scale(node_id, node.scale)
    }

    if !node.rotation.is_near_identity() {
        node_id = saft_graph_rotate(saft_graph, node_id, &node.rotation);
    };

    if node.translation.length_squared() != 0.0 {
        node_id = saft_graph_translate(saft_graph, node_id, &node.translation)
    }

    Some(node_id)
}

fn node_to_saft_node_data(
    saft_graph: &mut saft::Graph,
    graph: &Graph,
    node_data: &NodeData,
) -> Option<saft::NodeId> {
    match node_data {
        NodeData::Sphere(Sphere { radius }) => Some(saft_graph.sphere(glam::Vec3::ZERO, *radius)),
        NodeData::Cylinder(Cylinder {
            cylinder_radius,
            half_height,
            rounding_radius,
        }) => Some(saft_graph.rounded_cylinder(*cylinder_radius, *half_height, *rounding_radius)),
        NodeData::Torus(Torus { big_r, small_r }) => Some(saft_graph.torus(*big_r, *small_r)),

        NodeData::Union(Union {
            factor: size,
            children: nodes,
        }) => {
            let nodes: Vec<_> = nodes_to_saft_nodes(saft_graph, graph, nodes.as_slice());
            if nodes.is_empty() {
                return None;
            }
            if nodes.len() == 2 {
                let (lhs, rhs) = (nodes[0], nodes[1]);
                if *size == 0.0 {
                    Some(saft_graph.op_union(lhs, rhs))
                } else {
                    Some(saft_graph.op_union_smooth(lhs, rhs, *size))
                }
            } else if *size == 0.0 {
                Some(saft_graph.op_union_multi(nodes))
            } else {
                Some(saft_graph.op_union_multi_smooth(nodes, *size))
            }
        }
        NodeData::Intersect(Intersect {
            factor: size,
            children: nodes,
        }) => match lhs_rhs_to_saft_nodes(saft_graph, graph, nodes) {
            (Some(lhs), Some(rhs)) => {
                if *size == 0.0 {
                    Some(saft_graph.op_intersect(lhs, rhs))
                } else {
                    Some(saft_graph.op_intersect_smooth(lhs, rhs, *size))
                }
            }
            (Some(lhs), None) => Some(lhs),
            _ => None,
        },
        NodeData::Subtract(Subtract {
            factor: size,
            children: nodes,
        }) => {
            let nodes: Vec<_> = nodes_to_saft_nodes(saft_graph, graph, nodes.as_slice());
            if nodes.is_empty() {
                None
            } else if nodes.len() == 1 {
                Some(nodes[0])
            } else {
                let mut new_node_id = nodes[0];
                for rhs in &nodes[1..] {
                    if *size == 0.0 {
                        new_node_id = saft_graph.op_subtract(new_node_id, *rhs);
                    } else {
                        new_node_id = saft_graph.op_subtract_smooth(new_node_id, *rhs, *size);
                    }
                }
                Some(new_node_id)
            }
        }

        NodeData::Rgb(Rgb { rgb, child: node }) => {
            let child = node_to_saft_node(saft_graph, graph, (*node)?)?;
            Some(saft_graph.op_rgb(child, [rgb.0, rgb.1, rgb.2]))
        }

        NodeData::Translate(Translate {
            position,
            child: node,
        }) => {
            let child = node_to_saft_node(saft_graph, graph, (*node)?)?;
            Some(saft_graph_translate(saft_graph, child, position))
        }
        NodeData::Rotate(Rotate {
            rotation,
            child: node,
        }) => {
            let child = node_to_saft_node(saft_graph, graph, (*node)?)?;
            Some(saft_graph_rotate(saft_graph, child, rotation))
        }
        NodeData::Scale(Scale { scale, child: node }) => {
            let child = node_to_saft_node(saft_graph, graph, (*node)?)?;
            Some(saft_graph.op_scale(child, *scale))
        }
    }
}

fn nodes_to_saft_nodes(
    saft_graph: &mut saft::Graph,
    graph: &Graph,
    nodes: &[NodeId],
) -> Vec<saft::NodeId> {
    nodes
        .iter()
        .filter_map(|id| node_to_saft_node(saft_graph, graph, *id))
        .collect()
}

fn lhs_rhs_to_saft_nodes(
    saft_graph: &mut saft::Graph,
    graph: &Graph,
    nodes: &(Option<NodeId>, Option<NodeId>),
) -> (Option<saft::NodeId>, Option<saft::NodeId>) {
    (
        nodes
            .0
            .and_then(|node| node_to_saft_node(saft_graph, graph, node)),
        nodes
            .1
            .and_then(|node| node_to_saft_node(saft_graph, graph, node)),
    )
}

fn saft_graph_translate(
    graph: &mut saft::Graph,
    child: saft::NodeId,
    position: &Vec3,
) -> saft::NodeId {
    graph.op_translate(child, position.to_array())
}

fn saft_graph_rotate(
    graph: &mut saft::Graph,
    child: saft::NodeId,
    rotation: &Quat,
) -> saft::NodeId {
    graph.op_rotate(child, glam::Quat::from_array(rotation.to_array()))
}
