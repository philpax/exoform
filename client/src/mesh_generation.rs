use super::{CurrentEntity, RebuildTimer};
use bevy::prelude::*;
use shared::{
    Cylinder, Graph, Intersect, NodeData, NodeId, Rgb, Rotate, Scale, Sphere, Subtract, Torus,
    Translate, Union,
};

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

fn node_to_saft(graph: &Graph, root: NodeId) -> Option<(saft::Graph, saft::NodeId)> {
    let mut saft_graph = saft::Graph::default();
    let root_id = node_to_saft_node(&mut saft_graph, graph, root)?;
    Some((saft_graph, root_id))
}

fn sdf_to_bevy_mesh(graph: saft::Graph, root: saft::NodeId) -> Mesh {
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
            .map(|[r, g, b]| Color::rgb(r, g, b).as_linear_rgba_f32())
            .collect::<Vec<_>>(),
    );
    mesh.set_indices(Some(brm::Indices::U32(triangle_mesh.indices)));
    mesh
}

fn create_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    current_entity: &mut CurrentEntity,
    graph: &Graph,
) {
    let (saft_graph, root) = match node_to_saft(graph, graph.root_node_id) {
        Some(vals) => vals,
        None => return,
    };
    if saft_graph.bounding_box(root).volume() == 0.0 {
        return;
    }
    let mesh = sdf_to_bevy_mesh(saft_graph, root);

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

pub fn rebuild_mesh(
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
        &graph,
    );
}

pub fn keep_rebuilding_mesh(
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
