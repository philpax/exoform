use std::collections::HashSet;

use super::{render_egui_tree, SelectedNode};
use bevy::prelude::*;
use bevy_egui::egui;
use shared::{Graph, GraphEvent, Node, NodeData, NodeId};

pub fn coloured_button(text: &str, color: egui::color::Hsva) -> egui::Button {
    egui::widgets::Button::new(egui::RichText::new(text).color(color)).stroke(egui::Stroke {
        width: 2.0,
        color: color.into(),
    })
}

pub fn with_reset_button<T>(
    ui: &mut egui::Ui,
    mut value: T,
    default_value: T,
    main: impl FnOnce(&mut egui::Ui, &mut T) -> bool,
) -> Option<T> {
    ui.horizontal(|ui| {
        if main(ui, &mut value) {
            Some(value)
        } else if ui
            .small_button(egui::RichText::new("⟳").color(egui::Color32::WHITE))
            .clicked()
        {
            Some(default_value)
        } else {
            None
        }
    })
    .inner
}

pub fn grid<T>(ui: &mut egui::Ui, f: impl FnOnce(&mut egui::Ui) -> T) -> T {
    egui::Grid::new("rows")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, f)
        .inner
}

pub fn dragger_with_no_reset(ui: &mut egui::Ui, value: &mut f32) -> egui::Response {
    ui.add(
        egui::widgets::DragValue::new(value)
            .fixed_decimals(2)
            .speed(0.01),
    )
}

fn dragger(ui: &mut egui::Ui, value: f32, default_value: f32) -> Option<f32> {
    with_reset_button(ui, value, default_value, |ui, value| {
        dragger_with_no_reset(ui, value).changed()
    })
}

pub fn dragger_row(ui: &mut egui::Ui, label: &str, value: f32, default_value: f32) -> Option<f32> {
    with_label(ui, label, |ui| dragger(ui, value, default_value))
}

pub fn vec3(ui: &mut egui::Ui, value: Vec3, default_value: Vec3) -> Option<Vec3> {
    with_reset_button(ui, value, default_value, |ui, value| {
        ui.horizontal(|ui| {
            dragger_with_no_reset(ui, &mut value.x).changed()
                || dragger_with_no_reset(ui, &mut value.y).changed()
                || dragger_with_no_reset(ui, &mut value.z).changed()
        })
        .inner
    })
}

pub fn factor_slider(ui: &mut egui::Ui, value: f32, default_value: f32) -> Option<f32> {
    with_label(ui, "Factor", |ui| {
        with_reset_button(ui, value, default_value, |ui, value| {
            ui.add(egui::widgets::Slider::new(value, 0.0..=1.0))
                .changed()
        })
    })
}

pub fn factor_grid(
    ui: &mut egui::Ui,
    events: &mut Vec<GraphEvent>,
    node: &Node,
    value: f32,
    default_value: f32,
) -> Option<f32> {
    grid(ui, |ui| {
        events.extend(render_node_prelude_with_events(ui, node));
        factor_slider(ui, value, default_value)
    })
}

pub fn angle(ui: &mut egui::Ui, value: Quat, default_value: Quat) -> Option<Quat> {
    with_reset_button(ui, value, default_value, |ui, value| {
        let (mut yaw, mut pitch, mut roll) = value.to_euler(glam::EulerRot::YXZ);
        let response = ui.horizontal(|ui| {
            ui.drag_angle(&mut yaw).changed()
                || ui.drag_angle(&mut pitch).changed()
                || ui.drag_angle(&mut roll).changed()
        });
        *value = glam::Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll);
        response.inner
    })
}

pub fn colour(
    ui: &mut egui::Ui,
    value: (f32, f32, f32),
    default_value: (f32, f32, f32),
) -> Option<(f32, f32, f32)> {
    with_reset_button(ui, value, default_value, |ui, value| {
        let mut rgb = [value.0, value.1, value.2];
        let response = egui::widgets::color_picker::color_edit_button_rgb(ui, &mut rgb);
        [value.0, value.1, value.2] = rgb;

        response.changed()
    })
}

pub fn with_label<T>(ui: &mut egui::Ui, label: &str, f: impl Fn(&mut egui::Ui) -> T) -> T {
    ui.label(label);
    let result = f(ui);
    ui.end_row();
    result
}

pub fn render_transform(
    ui: &mut egui::Ui,
    transform: &shared::Transform,
) -> (Option<Vec3>, Option<Quat>, Option<f32>) {
    let tr = transform;
    (
        with_label(ui, "Translation", |ui| vec3(ui, tr.translation, Vec3::ZERO)),
        with_label(ui, "Rotation", |ui| angle(ui, tr.rotation, Quat::IDENTITY)),
        with_label(ui, "Scale", |ui| dragger(ui, tr.scale, 1.0)),
    )
}

pub fn render_colour_with_events(
    ui: &mut egui::Ui,
    node: &Node,
) -> impl Iterator<Item = GraphEvent> {
    let new_colour = with_label(ui, "Colour", |ui| {
        colour(ui, node.rgb, Node::DEFAULT_COLOUR)
    });

    new_colour
        .map(|rgb| GraphEvent::SetColour(node.id, rgb))
        .into_iter()
}

pub fn render_transform_with_events(
    ui: &mut egui::Ui,
    node: &Node,
) -> impl Iterator<Item = GraphEvent> {
    let (translation, rotation, scale) = render_transform(ui, &node.transform);
    let translation = translation.map(|t| GraphEvent::SetTranslation(node.id, t));
    let rotation = rotation.map(|r| GraphEvent::SetRotation(node.id, r));
    let scale = scale.map(|s| GraphEvent::SetScale(node.id, s));

    translation
        .into_iter()
        .chain(rotation.into_iter())
        .chain(scale.into_iter())
}

pub fn render_node_prelude_with_events(
    ui: &mut egui::Ui,
    node: &Node,
) -> impl Iterator<Item = GraphEvent> {
    render_colour_with_events(ui, node).chain(render_transform_with_events(ui, node))
}

pub fn render_add_dropdown(
    ui: &mut egui::Ui,
    response: egui::Response,
    include_primitives: bool,
) -> Option<NodeData> {
    ui.push_id(response.id, |ui| {
        let popup_id = ui.make_persistent_id("add_menu");
        if response.clicked() {
            ui.memory().toggle_popup(popup_id);
        }
        let mut new_node_data = None;
        egui::popup_below_widget(ui, popup_id, &response, |ui| {
            ui.set_min_width(200.0);
            for default in shared::NODE_DEFAULTS.iter() {
                let category_color = match default.category() {
                    shared::NodeCategory::Primitive => {
                        if !include_primitives {
                            continue;
                        }
                        egui::Color32::from_rgb(78, 205, 196)
                    }
                    shared::NodeCategory::Operation => egui::Color32::from_rgb(199, 244, 100),
                    shared::NodeCategory::Metadata => egui::Color32::from_rgb(255, 107, 107),
                    shared::NodeCategory::Transform => egui::Color32::from_rgb(238, 130, 238),
                };
                if ui
                    .add(egui::widgets::Button::new(
                        egui::RichText::new(default.name()).color(category_color),
                    ))
                    .clicked()
                {
                    new_node_data = Some(default.clone());
                }
            }
        });
        new_node_data
    })
    .inner
}

pub fn render_add_parent_button(ui: &mut egui::Ui, color: egui::color::Hsva) -> Option<NodeData> {
    let response = ui.add(coloured_button("⬆", color));
    render_add_dropdown(ui, response, false)
}

pub fn render_add_button_max_width(
    ui: &mut egui::Ui,
    color: egui::color::Hsva,
) -> Option<NodeData> {
    let response = ui.add_sized(
        egui::Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
        coloured_button("+", color),
    );
    render_add_dropdown(ui, response, true)
}

pub fn render_removable_trees(
    ui: &mut egui::Ui,
    graph: &Graph,
    selected_node: &mut SelectedNode,
    parent_id: NodeId,
    children: &[NodeId],
    depth: usize,
) -> impl Iterator<Item = GraphEvent> {
    let depth = depth + 1;
    let mut events = vec![];

    let mut to_remove = HashSet::new();
    for child_id in children {
        let (mut child_events, remove) =
            render_egui_tree(ui, graph, selected_node, Some(parent_id), *child_id, depth);
        events.append(&mut child_events);

        if remove {
            to_remove.insert(*child_id);
        }
    }
    events.extend(
        to_remove
            .into_iter()
            .map(move |child_id| GraphEvent::RemoveChild(parent_id, child_id)),
    );

    let new_child = render_add_button_max_width(ui, depth_to_color(depth, false));
    if let Some(node_data) = new_child {
        events.push(GraphEvent::AddChild(parent_id, None, node_data));
    }

    events.into_iter()
}

pub fn render_removable_tree_opt(
    ui: &mut egui::Ui,
    graph: &Graph,
    selected_node: &mut SelectedNode,
    parent_id: NodeId,
    child_id_opt: Option<NodeId>,
    child_index: usize,
    depth: usize,
) -> impl Iterator<Item = GraphEvent> {
    let depth = depth + 1;
    let mut events = vec![];

    match child_id_opt {
        Some(child_id) => {
            let (mut child_events, remove) =
                render_egui_tree(ui, graph, selected_node, Some(parent_id), child_id, depth);
            events.append(&mut child_events);

            if remove {
                events.push(GraphEvent::RemoveChild(parent_id, child_id));
            }
        }
        None => {
            let new_child = render_add_button_max_width(ui, depth_to_color(depth, false));
            if let Some(node_data) = new_child {
                events.push(GraphEvent::AddChild(
                    parent_id,
                    Some(child_index),
                    node_data,
                ));
            }
        }
    };

    events.into_iter()
}

pub fn depth_to_color(depth: usize, foreground: bool) -> egui::color::Hsva {
    let (s, v) = if foreground { (0.6, 0.8) } else { (0.9, 0.4) };
    egui::color::Hsva::new(((depth as f32 / 10.0) * 2.7) % 1.0, s, v, 1.0)
}