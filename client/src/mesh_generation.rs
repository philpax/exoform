use bevy::prelude::*;

struct CurrentEntity(Option<Entity>);
struct RebuildTimer(Timer);

pub struct MeshGenerationPlugin;
impl Plugin for MeshGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentEntity(None))
            .insert_resource(RebuildTimer(Timer::new(
                std::time::Duration::from_secs_f32(0.2),
                true,
            )))
            .add_startup_system(rebuild_mesh)
            .add_system(keep_rebuilding_mesh);
    }
}

fn rebuild_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_entity: ResMut<CurrentEntity>,
    graph: Res<shared::Graph>,
) {
    create_mesh(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut current_entity,
        &graph,
    );
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
