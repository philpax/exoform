use super::{CurrentEntity, Graph, RebuildTimer};
use bevy::prelude::*;
use shared::Node;

pub(crate) fn node_to_saft_node(graph: &mut saft::Graph, node: &Node) -> Option<saft::NodeId> {
    match node {
        Node::Sphere { position, radius } => Some(graph.sphere(*position, *radius)),
        Node::Cylinder {
            cylinder_radius,
            half_height,
            rounding_radius,
        } => Some(graph.rounded_cylinder(*cylinder_radius, *half_height, *rounding_radius)),
        Node::Torus { big_r, small_r } => Some(graph.torus(*big_r, *small_r)),

        Node::Union(size, nodes) => {
            let nodes: Vec<_> = nodes_to_saft_nodes(graph, nodes.as_slice());
            if nodes.is_empty() {
                return None;
            }
            if nodes.len() == 2 {
                let (lhs, rhs) = (nodes[0], nodes[1]);
                if *size == 0.0 {
                    Some(graph.op_union(lhs, rhs))
                } else {
                    Some(graph.op_union_smooth(lhs, rhs, *size))
                }
            } else if *size == 0.0 {
                Some(graph.op_union_multi(nodes))
            } else {
                Some(graph.op_union_multi_smooth(nodes, *size))
            }
        }
        Node::Intersect(size, nodes) => match lhs_rhs_to_saft_nodes(graph, nodes) {
            (Some(lhs), Some(rhs)) => {
                if *size == 0.0 {
                    Some(graph.op_intersect(lhs, rhs))
                } else {
                    Some(graph.op_intersect_smooth(lhs, rhs, *size))
                }
            }
            (Some(lhs), None) => Some(lhs),
            _ => None,
        },
        Node::Subtract(size, nodes) => match lhs_rhs_to_saft_nodes(graph, nodes) {
            (Some(lhs), Some(rhs)) => {
                if *size == 0.0 {
                    Some(graph.op_subtract(lhs, rhs))
                } else {
                    Some(graph.op_subtract_smooth(lhs, rhs, *size))
                }
            }
            (Some(lhs), None) => Some(lhs),
            _ => None,
        },

        Node::Rgb(r, g, b, node) => {
            let child = node_to_saft_node(graph, node.as_deref()?)?;
            Some(graph.op_rgb(child, [*r, *g, *b]))
        }

        Node::Translate(position, node) => {
            let child = node_to_saft_node(graph, node.as_deref()?)?;
            Some(graph.op_translate(child, position.to_array()))
        }
        Node::Rotate(rotation, node) => {
            let child = node_to_saft_node(graph, node.as_deref()?)?;
            Some(graph.op_rotate(child, glam::Quat::from_array(rotation.to_array())))
        }
        Node::Scale(scale, node) => {
            let child = node_to_saft_node(graph, node.as_deref()?)?;
            Some(graph.op_scale(child, *scale))
        }
    }
}

pub(crate) fn nodes_to_saft_nodes(graph: &mut saft::Graph, nodes: &[Node]) -> Vec<saft::NodeId> {
    nodes
        .iter()
        .filter_map(|n| node_to_saft_node(graph, n))
        .collect()
}

pub(crate) fn lhs_rhs_to_saft_nodes(
    graph: &mut saft::Graph,
    nodes: &(Option<Box<Node>>, Option<Box<Node>>),
) -> (Option<saft::NodeId>, Option<saft::NodeId>) {
    (
        nodes
            .0
            .as_ref()
            .and_then(|node| node_to_saft_node(graph, node)),
        nodes
            .1
            .as_ref()
            .and_then(|node| node_to_saft_node(graph, node)),
    )
}

pub(crate) fn node_to_saft(root: &Node) -> Option<(saft::Graph, saft::NodeId)> {
    let mut graph = saft::Graph::default();
    let root_id = node_to_saft_node(&mut graph, root)?;
    Some((graph, root_id))
}

pub(crate) fn create_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    current_entity: &mut CurrentEntity,
    root: &Node,
) {
    let (graph, root) = match node_to_saft(root) {
        Some(vals) => vals,
        None => return,
    };
    if graph.bounding_box(root).volume() == 0.0 {
        return;
    }
    let mesh = sdf_to_bevy_mesh(graph, root);

    if let Some(entity) = current_entity.0 {
        commands.entity(entity).despawn();
    }

    current_entity.0 = Some(
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::WHITE.into()),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            })
            .id(),
    );
}

pub(crate) fn sdf_to_bevy_mesh(graph: saft::Graph, root: saft::NodeId) -> Mesh {
    use bevy::render::mesh as brm;
    let triangle_mesh = saft::mesh_from_sdf(&graph, root, saft::MeshOptions::default()).unwrap();
    let mut mesh = Mesh::new(brm::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, triangle_mesh.normals);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        triangle_mesh
            .positions
            .iter()
            .map(|_| [0.0, 0.0])
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, triangle_mesh.positions);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        triangle_mesh
            .colors
            .into_iter()
            .map(|[r, g, b]| Color::rgb(r, g, b).as_rgba_u32())
            .collect::<Vec<_>>(),
    );
    mesh.set_indices(Some(brm::Indices::U32(triangle_mesh.indices)));
    mesh
}

pub(crate) fn rebuild_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_entity: ResMut<CurrentEntity>,
    graph: Res<Graph>,
) {
    create_mesh(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut current_entity,
        &graph.0,
    );
}

pub(crate) fn keep_rebuilding_mesh(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    current_entity: ResMut<CurrentEntity>,
    mut rebuild_timer: ResMut<RebuildTimer>,
    graph: Res<Graph>,
    time: Res<Time>,
) {
    rebuild_timer.0.tick(time.delta());
    if rebuild_timer.0.finished() {
        rebuild_mesh(commands, meshes, materials, current_entity, graph);
    }
}
