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

pub struct CompilationResult {
    pub mesh: Mesh,
    pub exo_node_count: usize,
    pub triangle_count: usize,
}

struct CompilationContext<'a> {
    saft_graph: &'a mut saft::Graph,
    exo_graph: &'a Graph,
    colours_enabled: bool,
}

pub fn generate_mesh(graph: &Graph, colours_enabled: bool) -> Option<CompilationResult> {
    let mut saft_graph = saft::Graph::default();
    let root_id = compile_node(
        &mut CompilationContext {
            saft_graph: &mut saft_graph,
            exo_graph: graph,
            colours_enabled,
        },
        graph.root_node_id()?,
    )?;

    let bounding_box = saft_graph.bounding_box(root_id);
    if bounding_box.volume() == 0.0 || !bounding_box.is_finite() {
        return None;
    }
    let mesh = saft::mesh_from_sdf(&saft_graph, root_id, saft::MeshOptions::default()).ok()?;
    let mesh = Mesh {
        indices: mesh.indices,
        positions: mesh.positions,
        normals: mesh.normals,
        colors: mesh.colors,
    };
    let triangle_count = mesh.indices.len() / 3;
    Some(CompilationResult {
        mesh,
        exo_node_count: graph.reachable_node_count(),
        triangle_count,
    })
}

fn compile_node(ctx: &mut CompilationContext, node: NodeId) -> Option<saft::NodeId> {
    let node = ctx.exo_graph.get(node).unwrap();
    let mut node_id = compile_node_data(ctx, &node.data, &node.children)?;
    let transform = &node.transform;
    if transform.scale != 1.0 {
        node_id = ctx.saft_graph.op_scale(node_id, transform.scale);
    }
    if !transform.rotation.is_near_identity() {
        node_id = saft_graph_rotate(ctx.saft_graph, node_id, &transform.rotation);
    }
    if transform.translation.length_squared() != 0.0 {
        node_id = saft_graph_translate(ctx.saft_graph, node_id, &transform.translation);
    }

    if ctx.colours_enabled && node.rgb != (1.0, 1.0, 1.0) {
        node_id = ctx
            .saft_graph
            .op_rgb(node_id, [node.rgb.0, node.rgb.1, node.rgb.2]);
    }

    Some(node_id)
}

fn compile_node_data(
    ctx: &mut CompilationContext,
    node_data: &NodeData,
    children: &[Option<NodeId>],
) -> Option<saft::NodeId> {
    match node_data {
        NodeData::Sphere(Sphere { radius }) => {
            Some(ctx.saft_graph.sphere(glam::Vec3::ZERO, *radius))
        }
        NodeData::Cylinder(Cylinder {
            cylinder_radius,
            half_height,
            rounding_radius,
        }) => Some(ctx.saft_graph.rounded_cylinder(
            *cylinder_radius,
            *half_height,
            *rounding_radius,
        )),
        NodeData::Torus(Torus { big_r, small_r }) => Some(ctx.saft_graph.torus(*big_r, *small_r)),
        NodeData::Plane(Plane {
            normal,
            distance_from_origin,
        }) => Some(
            ctx.saft_graph
                .plane((*normal, *distance_from_origin).into()),
        ),
        NodeData::Capsule(Capsule {
            point_1,
            point_2,
            radius,
        }) => Some(ctx.saft_graph.capsule([*point_1, *point_2], *radius)),
        NodeData::TaperedCapsule(TaperedCapsule {
            point_1,
            point_2,
            radius_1,
            radius_2,
        }) => Some(
            ctx.saft_graph
                .tapered_capsule([*point_1, *point_2], [*radius_1, *radius_2]),
        ),
        NodeData::Cone(Cone { radius, height }) => Some(ctx.saft_graph.cone(*radius, *height)),
        NodeData::Box(Box {
            half_size,
            rounding_radius,
        }) => Some(ctx.saft_graph.rounded_box(*half_size, *rounding_radius)),
        NodeData::TorusSector(TorusSector {
            big_r,
            small_r,
            angle,
        }) => Some(ctx.saft_graph.torus_sector(*big_r, *small_r, angle / 2.0)),
        NodeData::BiconvexLens(BiconvexLens {
            lower_sagitta,
            upper_sagitta,
            chord,
        }) => Some(
            ctx.saft_graph
                .biconvex_lens(*lower_sagitta, *upper_sagitta, *chord),
        ),

        NodeData::Union(Union { factor }) => {
            let nodes = compile_nodes(ctx, children);
            let is_unsmoothed = *factor == 0.0;
            if nodes.is_empty() {
                None
            } else if nodes.len() == 2 {
                let (lhs, rhs) = (nodes[0], nodes[1]);
                if is_unsmoothed {
                    Some(ctx.saft_graph.op_union(lhs, rhs))
                } else {
                    Some(ctx.saft_graph.op_union_smooth(lhs, rhs, *factor))
                }
            } else if is_unsmoothed {
                Some(ctx.saft_graph.op_union_multi(nodes))
            } else {
                Some(ctx.saft_graph.op_union_multi_smooth(nodes, *factor))
            }
        }
        NodeData::Intersect(Intersect { factor }) => {
            let nodes = compile_nodes(ctx, children);
            apply_infix_operation_over_array(&nodes, |lhs, rhs| {
                if *factor == 0.0 {
                    ctx.saft_graph.op_intersect(lhs, rhs)
                } else {
                    ctx.saft_graph.op_intersect_smooth(lhs, rhs, *factor)
                }
            })
        }
        NodeData::Subtract(Subtract { factor }) => {
            let nodes = compile_nodes(ctx, children);
            apply_infix_operation_over_array(&nodes, |lhs, rhs| {
                if *factor == 0.0 {
                    ctx.saft_graph.op_subtract(lhs, rhs)
                } else {
                    ctx.saft_graph.op_subtract_smooth(lhs, rhs, *factor)
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

fn compile_nodes(ctx: &mut CompilationContext, nodes: &[Option<NodeId>]) -> Vec<saft::NodeId> {
    nodes
        .iter()
        .filter_map(|id| compile_node(ctx, (*id)?))
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
