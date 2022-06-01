use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};

use smooth_bevy_cameras::{
    controllers::orbit::{
        ControlEvent, OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
    },
    LookTransformPlugin,
};

use shared::Node;

#[derive(Debug)]
pub struct Graph(Node);

pub struct CurrentEntity(Option<Entity>);

pub struct RebuildTimer(Timer);

fn build_sample_graph() -> Node {
    use Node::*;
    Union(
        0.0,
        vec![
            Subtract(
                0.0,
                (
                    Box::new(Intersect(
                        0.0,
                        (
                            Box::new(Subtract(
                                0.2,
                                (
                                    Box::new(Union(
                                        0.4,
                                        vec![
                                            Rgb(
                                                255.0,
                                                0.0,
                                                0.0,
                                                Box::new(Sphere {
                                                    position: Vec3::new(-0.5, 0.0, 0.0),
                                                    radius: 1.0,
                                                }),
                                            ),
                                            Rgb(
                                                0.0,
                                                255.0,
                                                0.0,
                                                Box::new(Sphere {
                                                    position: Vec3::new(0.0, 0.0, 0.5),
                                                    radius: 1.0,
                                                }),
                                            ),
                                            Rgb(
                                                0.0,
                                                0.0,
                                                255.0,
                                                Box::new(Sphere {
                                                    position: Vec3::new(0.5, 0.0, 0.0),
                                                    radius: 1.0,
                                                }),
                                            ),
                                        ],
                                    )),
                                    Box::new(Sphere {
                                        position: Vec3::new(0.0, 1.0, 0.0),
                                        radius: 0.6,
                                    }),
                                ),
                            )),
                            Box::new(Sphere {
                                position: Vec3::new(0.0, 0.0, 0.0),
                                radius: 1.2,
                            }),
                        ),
                    )),
                    Box::new(RoundedCylinder {
                        cylinder_radius: 0.5,
                        half_height: 2.0,
                        rounding_radius: 0.0,
                    }),
                ),
            ),
            Torus {
                big_r: 2.0,
                small_r: 0.5,
            },
        ],
    )
}

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .insert_resource(Graph(build_sample_graph()))
        .insert_resource(CurrentEntity(None))
        .insert_resource(RebuildTimer(Timer::new(
            std::time::Duration::from_secs(1),
            true,
        )))
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::new(true))
        .add_plugin(bevy_web_fullscreen::FullViewportPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(input_map)
        .add_system(sdf_code_editor)
        .add_system(keep_rebuilding_mesh)
        .run();
}

fn node_to_saft_node(graph: &mut saft::Graph, node: &Node) -> saft::NodeId {
    match node {
        Node::Sphere { position, radius } => graph.sphere(*position, *radius),
        Node::RoundedCylinder {
            cylinder_radius,
            half_height,
            rounding_radius,
        } => graph.rounded_cylinder(*cylinder_radius, *half_height, *rounding_radius),
        Node::Torus { big_r, small_r } => graph.torus(*big_r, *small_r),
        Node::Union(size, nodes) => {
            let nodes: Vec<_> = nodes_to_saft_nodes(graph, nodes.as_slice());
            if nodes.len() == 2 {
                let (lhs, rhs) = (nodes[0], nodes[1]);
                if *size == 0.0 {
                    graph.op_union(lhs, rhs)
                } else {
                    graph.op_union_smooth(lhs, rhs, *size)
                }
            } else if *size == 0.0 {
                graph.op_union_multi(nodes)
            } else {
                graph.op_union_multi_smooth(nodes, *size)
            }
        }
        Node::Intersect(size, nodes) => {
            let (lhs, rhs) = lhs_rhs_to_saft_nodes(graph, nodes);
            if *size == 0.0 {
                graph.op_intersect(lhs, rhs)
            } else {
                graph.op_intersect_smooth(lhs, rhs, *size)
            }
        }
        Node::Subtract(size, nodes) => {
            let (lhs, rhs) = lhs_rhs_to_saft_nodes(graph, nodes);
            if *size == 0.0 {
                graph.op_subtract(lhs, rhs)
            } else {
                graph.op_subtract_smooth(lhs, rhs, *size)
            }
        }
        Node::Rgb(r, g, b, node) => {
            let child = node_to_saft_node(graph, node);
            graph.op_rgb(child, [*r, *g, *b])
        }
    }
}

fn nodes_to_saft_nodes(graph: &mut saft::Graph, nodes: &[Node]) -> Vec<saft::NodeId> {
    nodes.iter().map(|n| node_to_saft_node(graph, n)).collect()
}

fn lhs_rhs_to_saft_nodes(
    graph: &mut saft::Graph,
    nodes: &(Box<Node>, Box<Node>),
) -> (saft::NodeId, saft::NodeId) {
    (
        node_to_saft_node(graph, &nodes.0),
        node_to_saft_node(graph, &nodes.1),
    )
}

fn node_to_saft(root: &Node) -> (saft::Graph, saft::NodeId) {
    let mut graph = saft::Graph::default();
    let root_id = node_to_saft_node(&mut graph, root);
    (graph, root_id)
}

fn create_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    current_entity: &mut CurrentEntity,
    root: &Node,
) {
    let (graph, root) = node_to_saft(root);
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

fn setup(mut commands: Commands) {
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn_bundle(OrbitCameraBundle::new(
        OrbitCameraController::default(),
        PerspectiveCameraBundle::default(),
        Vec3::new(-2.0, 5.0, 5.0),
        Vec3::new(0., 0., 0.),
    ));
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
            .map(|[r, g, b]| Color::rgb(r, g, b).as_rgba_u32())
            .collect::<Vec<_>>(),
    );
    mesh.set_indices(Some(brm::Indices::U32(triangle_mesh.indices)));
    mesh
}

pub fn input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut egui_context: ResMut<EguiContext>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    controllers: Query<&OrbitCameraController>,
) {
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    let egui_wants_input = {
        let ctx = egui_context.ctx_mut();
        ctx.wants_keyboard_input() || ctx.wants_pointer_input()
    };

    if !egui_wants_input && mouse_buttons.pressed(MouseButton::Left) {
        if keyboard.pressed(KeyCode::LControl) {
            events.send(ControlEvent::TranslateTarget(
                mouse_translate_sensitivity * cursor_delta,
            ));
        } else {
            events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
        }
    }
}

fn sdf_code_editor(mut egui_context: ResMut<EguiContext>, mut graph: ResMut<Graph>) {
    let ctx = egui_context.ctx_mut();
    egui::SidePanel::left("left_panel")
        .default_width(400.0)
        .show(ctx, |ui| {
            render_egui_tree(ui, &mut graph.0, 0);
        });
}

fn render_egui_tree(ui: &mut egui::Ui, node: &mut Node, index: usize) {
    let name = match node {
        Node::Sphere { .. } => "Sphere",
        Node::RoundedCylinder { .. } => "Cylinder",
        Node::Torus { .. } => "Torus",
        Node::Union(..) => "Union",
        Node::Intersect(..) => "Intersect",
        Node::Subtract(..) => "Subtract",
        Node::Rgb(..) => "Rgb",
    };

    fn dragger(value: &mut f32) -> egui::widgets::DragValue {
        egui::widgets::DragValue::new(value)
            .fixed_decimals(2)
            .speed(0.01)
    }

    fn vec3(ui: &mut egui::Ui, value: &mut Vec3) {
        ui.horizontal(|ui| {
            ui.add(dragger(&mut value.x));
            ui.add(dragger(&mut value.y));
            ui.add(dragger(&mut value.z));
        });
    }

    fn grid(ui: &mut egui::Ui, f: impl FnMut(&mut egui::Ui)) {
        egui::Grid::new("rows")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, f);
    }

    ui.push_id(index, |ui| {
        egui::CollapsingHeader::new(name)
            .default_open(true)
            .show(ui, |ui| match node {
                Node::Sphere { position, radius } => {
                    grid(ui, |ui| {
                        ui.label("Position");
                        vec3(ui, position);
                        ui.end_row();

                        ui.label("Radius");
                        ui.add(dragger(radius));
                        ui.end_row();
                    });
                }
                Node::RoundedCylinder {
                    cylinder_radius,
                    half_height,
                    rounding_radius,
                } => {
                    grid(ui, |ui| {
                        ui.label("Cylinder radius");
                        ui.add(dragger(cylinder_radius));
                        ui.end_row();

                        ui.label("Half height");
                        ui.add(dragger(half_height));
                        ui.end_row();

                        ui.label("Rounding radius");
                        ui.add(dragger(rounding_radius));
                        ui.end_row();
                    });
                }
                Node::Torus { big_r, small_r } => {
                    grid(ui, |ui| {
                        ui.label("Big radius");
                        ui.add(dragger(big_r));
                        ui.end_row();

                        ui.label("small height");
                        ui.add(dragger(small_r));
                        ui.end_row();
                    });
                }
                Node::Union(factor, children) => {
                    grid(ui, |ui| {
                        ui.label("Factor");
                        ui.add(dragger(factor).clamp_range(0.0..=1.0));
                        ui.end_row();
                    });
                    for (index, child) in children.iter_mut().enumerate() {
                        render_egui_tree(ui, child, index);
                    }
                }
                Node::Intersect(factor, (lhs, rhs)) => {
                    grid(ui, |ui| {
                        ui.label("Factor");
                        ui.add(dragger(factor).clamp_range(0.0..=1.0));
                        ui.end_row();
                    });
                    render_egui_tree(ui, lhs, 0);
                    render_egui_tree(ui, rhs, 1);
                }
                Node::Subtract(factor, (lhs, rhs)) => {
                    grid(ui, |ui| {
                        ui.label("Factor");
                        ui.add(dragger(factor).clamp_range(0.0..=1.0));
                        ui.end_row();
                    });
                    render_egui_tree(ui, lhs, 0);
                    render_egui_tree(ui, rhs, 1);
                }
                Node::Rgb(r, g, b, child) => {
                    grid(ui, |ui| {
                        ui.label("Colour");
                        let mut rgb = [*r, *g, *b];
                        egui::widgets::color_picker::color_edit_button_rgb(ui, &mut rgb);
                        [*r, *g, *b] = rgb;
                        ui.end_row();
                    });
                    render_egui_tree(ui, child, 0);
                }
            });
    });
}

fn keep_rebuilding_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_entity: ResMut<CurrentEntity>,
    mut rebuild_timer: ResMut<RebuildTimer>,
    graph: Res<Graph>,
    time: Res<Time>,
) {
    rebuild_timer.0.tick(time.delta());
    if rebuild_timer.0.finished() {
        create_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut current_entity,
            &graph.0,
        );
    }
}
