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

    let bounding_box = saft_graph.bounding_box(root_id);
    if bounding_box.volume() == 0.0 || !bounding_box.is_finite() {
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
    let mut node_id = node_to_saft_node_data(saft_graph, graph, &node.data, &node.children)?;
    let transform = &node.transform;
    if transform.scale != 1.0 {
        node_id = saft_graph.op_scale(node_id, transform.scale);
    }

    if !transform.rotation.is_near_identity() {
        node_id = saft_graph_rotate(saft_graph, node_id, &transform.rotation);
    }

    if transform.translation.length_squared() != 0.0 {
        node_id = saft_graph_translate(saft_graph, node_id, &transform.translation);
    }

    if node.rgb != (1.0, 1.0, 1.0) {
        node_id = saft_graph.op_rgb(node_id, [node.rgb.0, node.rgb.1, node.rgb.2]);
    }

    Some(node_id)
}

fn node_to_saft_node_data(
    saft_graph: &mut saft::Graph,
    graph: &Graph,
    node_data: &NodeData,
    children: &[Option<NodeId>],
) -> Option<saft::NodeId> {
    match node_data {
        NodeData::Sphere(Sphere { radius }) => Some(saft_graph.sphere(glam::Vec3::ZERO, *radius)),
        NodeData::Cylinder(Cylinder {
            cylinder_radius,
            half_height,
            rounding_radius,
        }) => Some(saft_graph.rounded_cylinder(*cylinder_radius, *half_height, *rounding_radius)),
        NodeData::Torus(Torus { big_r, small_r }) => Some(saft_graph.torus(*big_r, *small_r)),
        NodeData::Plane(Plane {
            normal,
            distance_from_origin,
        }) => Some(saft_graph.plane((*normal, *distance_from_origin).into())),
        NodeData::Capsule(Capsule {
            point_1,
            point_2,
            radius,
        }) => Some(saft_graph.capsule([*point_1, *point_2], *radius)),
        NodeData::TaperedCapsule(TaperedCapsule {
            point_1,
            point_2,
            radius_1,
            radius_2,
        }) => Some(saft_graph.tapered_capsule([*point_1, *point_2], [*radius_1, *radius_2])),
        NodeData::Cone(Cone { radius, height }) => Some(saft_graph.cone(*radius, *height)),
        NodeData::Box(Box {
            half_size,
            rounding_radius,
        }) => Some(saft_graph.rounded_box(*half_size, *rounding_radius)),
        NodeData::TorusSector(TorusSector {
            big_r,
            small_r,
            angle,
        }) => Some(saft_graph.torus_sector(*big_r, *small_r, angle / 2.0)),
        NodeData::BiconvexLens(BiconvexLens {
            lower_sagitta,
            upper_sagitta,
            chord,
        }) => Some(saft_graph.biconvex_lens(*lower_sagitta, *upper_sagitta, *chord)),

        NodeData::Union(Union { factor }) => {
            let nodes = nodes_to_saft_nodes(saft_graph, graph, children);
            let is_unsmoothed = *factor == 0.0;
            if nodes.is_empty() {
                None
            } else if nodes.len() == 2 {
                let (lhs, rhs) = (nodes[0], nodes[1]);
                if is_unsmoothed {
                    Some(saft_graph.op_union(lhs, rhs))
                } else {
                    Some(saft_graph.op_union_smooth(lhs, rhs, *factor))
                }
            } else if is_unsmoothed {
                Some(saft_graph.op_union_multi(nodes))
            } else {
                Some(saft_graph.op_union_multi_smooth(nodes, *factor))
            }
        }
        NodeData::Intersect(Intersect { factor }) => {
            let nodes = nodes_to_saft_nodes(saft_graph, graph, children);
            apply_infix_operation_over_array(&nodes, |lhs, rhs| {
                if *factor == 0.0 {
                    saft_graph.op_intersect(lhs, rhs)
                } else {
                    saft_graph.op_intersect_smooth(lhs, rhs, *factor)
                }
            })
        }
        NodeData::Subtract(Subtract { factor }) => {
            let nodes = nodes_to_saft_nodes(saft_graph, graph, children);
            apply_infix_operation_over_array(&nodes, |lhs, rhs| {
                if *factor == 0.0 {
                    saft_graph.op_subtract(lhs, rhs)
                } else {
                    saft_graph.op_subtract_smooth(lhs, rhs, *factor)
                }
            })
        }
    }
}

fn apply_infix_operation_over_array(
    nodes: &[saft::NodeId],
    mut operation: impl FnMut(saft::NodeId, saft::NodeId) -> saft::NodeId,
) -> Option<saft::NodeId> {
    if nodes.is_empty() {
        None
    } else if nodes.len() == 1 {
        Some(nodes[0])
    } else {
        let mut new_node_id = nodes[0];
        for rhs in &nodes[1..] {
            new_node_id = operation(new_node_id, *rhs);
        }
        Some(new_node_id)
    }
}

fn nodes_to_saft_nodes(
    saft_graph: &mut saft::Graph,
    graph: &Graph,
    nodes: &[Option<NodeId>],
) -> Vec<saft::NodeId> {
    nodes
        .iter()
        .filter_map(|id| node_to_saft_node(saft_graph, graph, (*id)?))
        .collect()
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
