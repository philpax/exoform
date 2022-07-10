use std::collections::HashSet;

use bevy::prelude::*;

struct CurrentEntity(Option<Entity>);
struct RebuildTimer(Timer);

#[derive(Component)]
struct GeneratedMesh(shared::NodeId);
struct GeneratedRootId(Entity);

pub struct MeshGenerationPlugin;
impl Plugin for MeshGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentEntity(None))
            .insert_resource(RebuildTimer(Timer::new(
                std::time::Duration::from_secs_f32(0.2),
                true,
            )))
            .add_startup_system(prepare_initial_generated_root)
            .add_startup_system(rebuild_mesh)
            .add_system(keep_rebuilding_mesh);
    }
}

fn prepare_initial_generated_root(
    mut commands: Commands,
    mut generated_root_id: ResMut<GeneratedRootId>,
) {
    generated_root_id.0 = commands.spawn().id();
}

fn rebuild_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_entity: ResMut<CurrentEntity>,
    graph: Res<shared::Graph>,
    // query: Query<(Entity, &Transform, &GeneratedMesh)>,
) {
    create_mesh(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut current_entity,
        &graph,
    );

    let mut reachable_nodes = HashSet::new();
    graph.find_all_reachable_nodes(graph.root_node_id, &mut reachable_nodes);
    // add the nodes that are in the reachable nodes and not present in the Bevy set
    // remove the nodes that are in the Bevy set, but not in the reachable nodes
    // ensure the parenting relationship is correct for each Bevy node
    // move the Bevy set to their transforms in the graph
    // every 50ms, send an update to synchronise the graph with the transforms from the Bevy nodes
}

fn keep_rebuilding_mesh(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    current_entity: ResMut<CurrentEntity>,
    mut rebuild_timer: ResMut<RebuildTimer>,
    graph: Res<shared::Graph>,
    time: Res<Time>,
) {
    rebuild_timer.0.tick(time.delta());
    if rebuild_timer.0.finished() {
        rebuild_mesh(commands, meshes, materials, current_entity, graph);
    }
}

fn create_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    current_entity: &mut CurrentEntity,
    graph: &shared::Graph,
) {
    let raw_mesh = match shared::mesh::generate_mesh(graph) {
        Some(m) => m,
        None => return,
    };
    let mesh = convert_to_bevy_mesh(raw_mesh);

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

fn convert_to_bevy_mesh(raw_mesh: shared::mesh::Mesh) -> Mesh {
    use bevy::render::mesh as brm;

    let uvs = std::iter::repeat([0.0, 0.0])
        .take(raw_mesh.positions.len())
        .collect::<Vec<_>>();
    let colours = raw_mesh
        .colors
        .into_iter()
        .map(|[r, g, b]| Color::rgb(r, g, b).as_linear_rgba_f32())
        .collect::<Vec<_>>();

    let mut mesh = Mesh::new(brm::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, raw_mesh.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, raw_mesh.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    mesh.set_indices(Some(brm::Indices::U32(raw_mesh.indices)));
    mesh
}

fn shared_transform_to_bevy_transform(t: shared::Transform) -> Transform {
    Transform {
        translation: t.translation,
        rotation: t.rotation,
        scale: Vec3::ONE * t.scale,
    }
}

fn node_to_generated_mesh(
    commands: &mut ChildBuilder,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    graph: &shared::Graph,
    node_id: shared::NodeId,
) {
    use shared::{Intersect, NodeData, Subtract, Union};

    let node = graph.get(node_id).unwrap();
    let (r, g, b) = node.rgb;

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.2 * node.transform.scale,
                ..Default::default()
            })),
            material: materials.add(Color::rgb(r, g, b).into()),
            transform: shared_transform_to_bevy_transform(node.transform.clone()),
            ..Default::default()
        })
        .insert_bundle(bevy_mod_picking::PickableBundle::default())
        .insert(bevy_transform_gizmo::GizmoTransformable)
        .insert(GeneratedMesh(node_id))
        .with_children(|child_commands| match &node.data {
            NodeData::Sphere(_) => {}
            NodeData::Cylinder(_) => {}
            NodeData::Torus(_) => {}
            NodeData::Plane(_) => {}
            NodeData::Capsule(_) => {}
            NodeData::TaperedCapsule(_) => {}
            NodeData::Cone(_) => {}
            NodeData::Box(_) => {}
            NodeData::TorusSector(_) => {}
            NodeData::BiconvexLens(_) => {}

            NodeData::Union(Union { children, .. }) => {
                for child_id in children {
                    node_to_generated_mesh(child_commands, meshes, materials, graph, *child_id);
                }
            }
            NodeData::Intersect(Intersect {
                children: (lhs, rhs),
                ..
            }) => {
                if let Some(lhs) = lhs {
                    node_to_generated_mesh(child_commands, meshes, materials, graph, *lhs);
                }
                if let Some(rhs) = rhs {
                    node_to_generated_mesh(child_commands, meshes, materials, graph, *rhs);
                }
            }
            NodeData::Subtract(Subtract { children, .. }) => {
                for child_id in children {
                    node_to_generated_mesh(child_commands, meshes, materials, graph, *child_id);
                }
            }
        });
}
