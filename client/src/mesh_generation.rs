use bevy::prelude::*;

use crate::resources::{MeshGenerationResult, RenderParameters};

struct CurrentEntity(Option<Entity>);

pub struct MeshGenerationPlugin;
impl Plugin for MeshGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentEntity(None))
            .add_system(keep_rebuilding_mesh);
    }
}

fn keep_rebuilding_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_entity: ResMut<CurrentEntity>,
    mut mesh_generation_result: ResMut<MeshGenerationResult>,
    render_parameters: Res<RenderParameters>,
    graph: Res<shared::Graph>,
) {
    if !(render_parameters.is_changed() || graph.is_added() || graph.is_changed()) {
        return;
    }

    if let Some(entity) = current_entity.0.take() {
        commands.entity(entity).despawn();
    }

    let mesh = match shared::mesh::generate_mesh(&graph, render_parameters.colours) {
        Ok(result) => {
            *mesh_generation_result = MeshGenerationResult::Successful {
                triangle_count: result.triangle_count,
                volume: result.volume,
            };
            result.mesh
        }
        Err(err) => {
            *mesh_generation_result = MeshGenerationResult::Failure(err);
            return;
        }
    };
    let mesh = convert_to_bevy_mesh(mesh);

    let mut spawn_bundle = commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
    if render_parameters.wireframe {
        spawn_bundle.insert(bevy::pbr::wireframe::Wireframe);
    }
    current_entity.0 = Some(spawn_bundle.id());
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
