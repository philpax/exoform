use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_egui::{egui, EguiContext, EguiPlugin};

use smooth_bevy_cameras::{
    controllers::orbit::{
        ControlEvent, OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
    },
    LookTransformPlugin,
};

use shared::canonical as cn;

mod ui;

use ui::graph as uig;

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut graph = cn::Graph::new();
    let sphere_1_id = graph.add_node(
        cn::NodeData::sphere(Vec3::new(0.0, 0.0, 0.0), 4.0),
        Vec2::new(10.0, 10.0),
    );
    let sphere_2_id = graph.add_node(
        cn::NodeData::sphere(Vec3::new(6.0, 0.0, 0.0), 4.0),
        Vec2::new(10.0, 210.0),
    );
    let union_id = graph.add_node(cn::NodeData::Union, Vec2::new(250.0, 105.0));
    graph.connect_by_ids((sphere_1_id, 0), (union_id, 0));
    graph.connect_by_ids((sphere_2_id, 0), (union_id, 1));
    let output_id = graph.add_node(cn::NodeData::Output, Vec2::new(280.0, 10.0));
    graph.connect_by_ids((union_id, 0), (output_id, 0));

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::new(true))
        .add_plugin(bevy_web_fullscreen::FullViewportPlugin)
        .add_plugin(EguiPlugin)
        .insert_resource(uig::EditorState::new(1.0, uig::GraphState::default()))
        .insert_resource(graph)
        .add_startup_system(setup)
        .add_system(input_map)
        .add_system(sync_graphs)
        .add_system(render_ui_graph)
        .add_system(render_canonical_state)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
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

fn sync_graphs(mut sdf_editor_state: ResMut<uig::EditorState>, mut graph: ResMut<cn::Graph>) {
    let mut responses = vec![];
    responses.append(&mut sdf_editor_state.user_state.node_responses);
    for response in responses {
        uig::sync::sync_ui_to_canonical(&response, &mut sdf_editor_state, &mut graph);
    }

    for event in graph.drain_arrived_incoming_events() {
        uig::sync::sync_canonical_to_ui(event, &mut sdf_editor_state, &graph);
    }

    // Once the bidirectional sync is complete, ensure that every node's IO is linked up correctly.
    // TODO(philpax): get rid of this! We should detect when a UI interaction has resulted in a change
    // to the canonical state, and then link the UI and canonical representation together then,
    // not every tick...
    let ids: Vec<_> = sdf_editor_state
        .user_state
        .node_map
        .iter()
        .map(|(c, u)| (*c, *u))
        .collect();

    for (canonical_node_id, ui_node_id) in ids {
        uig::sync::link_canonical_and_ui_io(
            &graph,
            &mut sdf_editor_state,
            canonical_node_id,
            ui_node_id,
        );
    }
}

fn render_ui_graph(
    mut egui_context: ResMut<EguiContext>,
    mut sdf_editor_state: ResMut<uig::EditorState>,
) {
    let ctx = egui_context.ctx_mut();
    egui::Window::new("UI Graph").show(ctx, |ui| {
        ui.horizontal(|ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Build").clicked() {
                    //
                };
            });
        });

        ui.with_layout(
            egui::Layout::top_down_justified(egui::Align::Center),
            |ui| {
                let node_response =
                    sdf_editor_state.draw_graph_editor(ctx, ui, uig::NodeTemplatesAll);

                sdf_editor_state
                    .user_state
                    .node_responses
                    .extend_from_slice(&node_response.node_responses);
            },
        );
    });
}

fn render_canonical_state(mut egui_context: ResMut<EguiContext>, graph: Res<cn::Graph>) {
    let ctx = egui_context.ctx_mut();
    egui::Window::new("Canonical State").show(ctx, |ui| {
        ui.heading("nodes");
        for node in graph.nodes().values() {
            ui.label(&format!("{:?}", node));
        }

        ui.heading("edges");
        for edge in graph.edges() {
            ui.label(&format!("{:?}", edge));
        }

        ui.heading("incoming_events");
        for event in graph.incoming_events() {
            ui.label(&format!("{:?}", event));
        }
    });
}
