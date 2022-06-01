use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};

use shared::Node;

#[derive(Default)]
struct OccupiedScreenSpace {
    left: f32,
    _top: f32,
    _right: f32,
    _bottom: f32,
}
#[derive(Debug)]
struct Graph(Node);
struct CurrentEntity(Option<Entity>);
struct RebuildTimer(Timer);

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

    let mut app = App::new();
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .insert_resource(Graph(build_sample_graph()))
        .insert_resource(CurrentEntity(None))
        .insert_resource(RebuildTimer(Timer::new(
            std::time::Duration::from_secs(1),
            true,
        )))
        .insert_resource(WindowDescriptor {
            width: 1600.,
            height: 900.,
            ..Default::default()
        })
        .init_resource::<OccupiedScreenSpace>()
        .add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_web_fullscreen::FullViewportPlugin);

    app.add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(sdf_code_editor)
        .add_system(keep_rebuilding_mesh)
        .add_system(pan_orbit_camera)
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
    let eye = Vec3::new(-2.0, 5.0, 5.0);
    let target = Vec3::new(0., 0., 0.);
    let transform = Transform::from_translation(eye).looking_at(target, Vec3::Y);
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform,
            ..Default::default()
        })
        .insert(PanOrbitCamera {
            radius: eye.distance(target),
            ..Default::default()
        });
}

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

// Based off
// - https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
// - https://github.com/mvlabat/bevy_egui/blob/main/examples/side_panel.rs
// (thank you!)
fn pan_orbit_camera(
    occupied_screen_space: Res<OccupiedScreenSpace>,
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    input_mouse: Res<Input<MouseButton>>,
    mut egui_context: ResMut<EguiContext>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &PerspectiveProjection)>,
) {
    let orbit_button = MouseButton::Left;
    let pan_button = MouseButton::Middle;
    let zoom_button = MouseButton::Right;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut zoom = 0.0;
    let mut orbit_button_changed = false;

    let egui_wants_input = {
        let ctx = egui_context.ctx_mut();
        ctx.wants_keyboard_input() || ctx.wants_pointer_input()
    };
    if !egui_wants_input {
        if input_mouse.pressed(orbit_button) {
            for ev in ev_motion.iter() {
                rotation_move += ev.delta;
            }
        } else if input_mouse.pressed(pan_button) {
            // Pan only if we're not rotating at the moment
            for ev in ev_motion.iter() {
                pan += ev.delta;
            }
        } else if input_mouse.pressed(zoom_button) {
            for ev in ev_motion.iter() {
                zoom += ev.delta.x;
            }
        }
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    let window = get_primary_window_size(&windows);
    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        if rotation_move.length_squared() > 0.0 {
            let (yaw, pitch) = {
                let delta_x = {
                    let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                    if pan_orbit.upside_down {
                        -delta
                    } else {
                        delta
                    }
                };
                let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
                (
                    Quat::from_rotation_y(-delta_x),
                    Quat::from_rotation_x(-delta_y),
                )
            };
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation *= pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            // make panning distance independent of resolution and FOV,
            let pan =
                pan * Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if zoom.abs() > 0.0 {
            let zoom = zoom * projection.fov * projection.aspect_ratio / window.x;
            pan_orbit.radius -= zoom * pan_orbit.radius;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
        // parent = x and y rotation
        // child = z-offset
        let rot_matrix = Mat3::from_quat(transform.rotation);
        let uncorrected_translation =
            pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));

        // Once the initial translation has been calculated, add in an offset to handle the
        // complications from having a side panel.
        let frustum_height = 2.0 * pan_orbit.radius * (projection.fov * 0.5).tan();
        let frustum_width = frustum_height * projection.aspect_ratio;

        let window = windows.get_primary().unwrap();

        let left_taken = occupied_screen_space.left / window.width();
        let right_taken = occupied_screen_space._right / window.width();
        let top_taken = occupied_screen_space._top / window.height();
        let bottom_taken = occupied_screen_space._bottom / window.height();
        let offset = transform.rotation.mul_vec3(Vec3::new(
            (right_taken - left_taken) * frustum_width * 0.5,
            (top_taken - bottom_taken) * frustum_height * 0.5,
            0.0,
        ));
        transform.translation = uncorrected_translation + offset;
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    Vec2::new(window.width() as f32, window.height() as f32)
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

fn sdf_code_editor(
    mut egui_context: ResMut<EguiContext>,
    mut graph: ResMut<Graph>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = egui_context.ctx_mut();
    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .default_width(400.0)
        .show(ctx, |ui| {
            render_egui_tree(ui, &mut graph.0, 0);
        })
        .response
        .rect
        .width();
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

    fn factor_slider(ui: &mut egui::Ui, factor: &mut f32) {
        grid(ui, |ui| {
            ui.label("Factor");
            ui.add(egui::widgets::Slider::new(factor, 0.0..=1.0));
            ui.end_row();
        });
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

                        ui.label("Small radius");
                        ui.add(dragger(small_r));
                        ui.end_row();
                    });
                }
                Node::Union(factor, children) => {
                    factor_slider(ui, factor);
                    for (index, child) in children.iter_mut().enumerate() {
                        render_egui_tree(ui, child, index);
                    }
                }
                Node::Intersect(factor, (lhs, rhs)) => {
                    factor_slider(ui, factor);
                    render_egui_tree(ui, lhs, 0);
                    render_egui_tree(ui, rhs, 1);
                }
                Node::Subtract(factor, (lhs, rhs)) => {
                    factor_slider(ui, factor);
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
