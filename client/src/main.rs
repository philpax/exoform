use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};

use smooth_bevy_cameras::{
    controllers::orbit::{
        ControlEvent, OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
    },
    LookTransformPlugin,
};

use shared::{code_to_node, Node};

#[derive(Debug, PartialEq)]
pub struct GraphDescription(String);

#[derive(Debug)]
pub struct ParsedGraph(anyhow::Result<Node>);

pub struct CurrentEntity(Option<Entity>);

pub struct RebuildTimer(Timer);

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let description_base = r#"
union {
    subtract {
        intersect {
            subtract 0.2 {
                union 0.4 {
                    rgb 255 0 0 {
                        sphere -0.5 0 0 1
                    }
                    rgb 0 255 0 {
                        sphere 0 0 0.5 1
                    }
                    rgb 0 0 255 {
                        sphere 0.5 0 0 1
                    }
                }
                sphere 0 1 0 0.6
            }
            sphere 0 0 0 1.2
        }
        cylinder 0.5 2.0 0
    }

    torus 2.0 0.5
}
"#;

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .insert_resource(GraphDescription(description_base.trim().to_string()))
        .insert_resource(ParsedGraph(Err(anyhow::anyhow!("unparsed graph"))))
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
        .add_system(parse_code)
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
            } else {
                if *size == 0.0 {
                    graph.op_union_multi(nodes)
                } else {
                    graph.op_union_multi_smooth(nodes, *size)
                }
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
            let child = node_to_saft_node(graph, &node);
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

fn parse_code(mut parsed_graph: ResMut<ParsedGraph>, graph_description: Res<GraphDescription>) {
    *parsed_graph = ParsedGraph(code_to_node(&graph_description.0));
}

fn sdf_code_editor(
    mut egui_context: ResMut<EguiContext>,
    mut graph_description: ResMut<GraphDescription>,
    parsed_graph: Res<ParsedGraph>,
) {
    let ctx = egui_context.ctx_mut();

    egui::SidePanel::left("left_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(false)
                .show_inside(ui, |ui| {
                    if let Err(err) = &parsed_graph.0 {
                        ui.label(err.to_string());
                    }
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut graph_description.0)
                            .font(egui::TextStyle::Monospace) // for cursor height
                            .code_editor()
                            .lock_focus(true)
                            .desired_width(f32::INFINITY),
                    );
                });
            });
        });

    egui::SidePanel::right("right_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            if let Ok(node) = &parsed_graph.0 {
                render_egui_tree(ui, node);
            }
        });
}

fn render_egui_tree(ui: &mut egui::Ui, node: &Node) {
    let (name, children) = match node {
        Node::Sphere { position, radius } => (format!("Sphere({position}, {radius})"), vec![]),
        Node::RoundedCylinder {
            cylinder_radius,
            half_height,
            rounding_radius,
        } => (
            format!("Cylinder({cylinder_radius}, {half_height}, {rounding_radius})"),
            vec![],
        ),
        Node::Torus { big_r, small_r } => (format!("Torus({big_r}, {small_r})"), vec![]),

        Node::Union(size, children) => (format!("Union({size})"), children.iter().collect()),
        Node::Intersect(size, (lhs, rhs)) => (
            format!("Intersect({size})"),
            vec![lhs.as_ref(), rhs.as_ref()],
        ),
        Node::Subtract(size, (lhs, rhs)) => (
            format!("Subtract({size})"),
            vec![lhs.as_ref(), rhs.as_ref()],
        ),

        Node::Rgb(r, g, b, child) => (format!("RGB({r}, {g}, {b})"), vec![child.as_ref()]),
    };

    if children.is_empty() {
        ui.label(name);
    } else {
        egui::CollapsingHeader::new(name)
            .default_open(true)
            .show(ui, |ui| {
                for child in children {
                    render_egui_tree(ui, child);
                }
            });
    }
}

fn keep_rebuilding_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_entity: ResMut<CurrentEntity>,
    mut rebuild_timer: ResMut<RebuildTimer>,
    parsed_graph: Res<ParsedGraph>,
    time: Res<Time>,
) {
    rebuild_timer.0.tick(time.delta());
    if let Ok(root) = &parsed_graph.0 {
        if rebuild_timer.0.finished() {
            create_mesh(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut current_entity,
                root,
            );
        }
    }
}
