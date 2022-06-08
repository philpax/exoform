use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use super::OccupiedScreenSpace;
use shared::{
    Cylinder, Graph, GraphEvent, Intersect, NodeData, NodeId, Sphere, Subtract, Torus, Union,
};

pub fn sdf_code_editor(
    mut egui_context: ResMut<EguiContext>,
    mut graph: ResMut<Graph>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = egui_context.ctx_mut();

    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        .show(ctx, |ui| {
            egui::menu::bar(ui, |_ui| {});
        })
        .response
        .rect
        .height();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .default_width(400.0)
        .show(ctx, |ui| {
            let node_id = graph.root_node_id;
            let (events, _) = render_egui_tree(ui, &graph, None, node_id, 0);
            graph.apply_events(&events);
        })
        .response
        .rect
        .width();

    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .default_width(400.0)
        .show(ctx, |_ui| {})
        .response
        .rect
        .width();
}

fn render_header(
    ui: &mut egui::Ui,
    graph: &Graph,
    node_id: NodeId,
    color: egui::color::Hsva,
    remove: &mut bool,
) {
    let node = graph.get(node_id).unwrap();
    ui.label(
        egui::RichText::new(node.data.name())
            .color(color)
            .text_style(egui::TextStyle::Monospace),
    );

    ui.add_space(ui.available_width() - ui.spacing().interact_size.x);

    if ui
        .small_button(egui::RichText::new("X").color(egui::Color32::LIGHT_RED))
        .clicked()
    {
        *remove = true;
    }
}

fn render_body(
    ui: &mut egui::Ui,
    graph: &Graph,
    parent_node_id: Option<NodeId>,
    node_id: NodeId,
    depth: usize,
) -> Vec<GraphEvent> {
    let mut events = vec![];

    if let Some(parent_node_id) = parent_node_id {
        if let Some(node_data) =
            util::render_add_button(ui, "Add Parent", false, util::depth_to_color(depth))
        {
            events.push(GraphEvent::AddNewParent(parent_node_id, node_id, node_data));
        }
    }

    let node = graph.get(node_id).unwrap();
    match &node.data {
        NodeData::Sphere(Sphere { radius }) => {
            let default = Sphere::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                if let Some(radius) = util::dragger_row(ui, "Radius", *radius, default.radius) {
                    events.push(GraphEvent::ReplaceData(
                        node_id,
                        NodeData::Sphere(Sphere { radius }),
                    ));
                }
            });
        }
        NodeData::Cylinder(Cylinder {
            cylinder_radius,
            half_height,
            rounding_radius,
        }) => {
            let default = Cylinder::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_cylinder_radius = util::dragger_row(
                    ui,
                    "Cylinder radius",
                    *cylinder_radius,
                    default.cylinder_radius,
                );
                let new_half_height =
                    util::dragger_row(ui, "Half height", *half_height, default.half_height);
                let new_rounding_radius = util::dragger_row(
                    ui,
                    "Rounding radius",
                    *rounding_radius,
                    default.rounding_radius,
                );

                match (new_cylinder_radius, new_half_height, new_rounding_radius) {
                    (None, None, None) => {}
                    (cy, ha, rr) => events.push(GraphEvent::ReplaceData(
                        node_id,
                        NodeData::Cylinder(Cylinder {
                            cylinder_radius: cy.unwrap_or(*cylinder_radius),
                            half_height: ha.unwrap_or(*half_height),
                            rounding_radius: rr.unwrap_or(*rounding_radius),
                        }),
                    )),
                }
            });
        }
        NodeData::Torus(Torus { big_r, small_r }) => {
            let default = Torus::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_big_r = util::dragger_row(ui, "Big radius", *big_r, default.big_r);
                let new_small_r = util::dragger_row(ui, "Small radius", *small_r, default.small_r);

                match (new_big_r, new_small_r) {
                    (None, None) => {}
                    (br, sr) => events.push(GraphEvent::ReplaceData(
                        node_id,
                        NodeData::Torus(Torus {
                            big_r: br.unwrap_or(*big_r),
                            small_r: sr.unwrap_or(*small_r),
                        }),
                    )),
                }
            });
        }

        NodeData::Union(Union { factor, children }) => {
            let default = Union::default();
            let new_factor = util::factor_grid(ui, &mut events, node, *factor, default.factor);
            if let Some(factor) = new_factor {
                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::Union(Union {
                        factor,
                        children: children.clone(),
                    }),
                ))
            }

            events.extend(util::render_removable_trees(
                ui, graph, node_id, children, depth,
            ));
        }
        NodeData::Intersect(Intersect {
            factor,
            children: (lhs, rhs),
        }) => {
            let default = Intersect::default();
            let new_factor = util::factor_grid(ui, &mut events, node, *factor, default.factor);
            if let Some(factor) = new_factor {
                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::Intersect(Intersect {
                        factor,
                        children: (*lhs, *rhs),
                    }),
                ))
            }

            events.extend(util::render_removable_tree_opt(
                ui, graph, node_id, *lhs, 0, depth,
            ));
            events.extend(util::render_removable_tree_opt(
                ui, graph, node_id, *rhs, 1, depth,
            ));
        }
        NodeData::Subtract(Subtract { factor, children }) => {
            let default = Subtract::default();
            let new_factor = util::factor_grid(ui, &mut events, node, *factor, default.factor);
            if let Some(factor) = new_factor {
                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::Subtract(Subtract {
                        factor,
                        children: children.clone(),
                    }),
                ))
            }

            events.extend(util::render_removable_trees(
                ui, graph, node_id, children, depth,
            ));
        }
    }

    events
}

pub fn render_egui_tree(
    ui: &mut egui::Ui,
    graph: &Graph,
    parent_node_id: Option<NodeId>,
    node_id: NodeId,
    depth: usize,
) -> (Vec<GraphEvent>, bool) {
    let color = util::depth_to_color(depth);
    let name = graph.get(node_id).unwrap().data.name().to_owned();

    let mut remove = false;
    let events = ui
        .push_id(node_id, |ui| {
            egui::Frame::none()
                .stroke(egui::Stroke::new(1.0, color))
                .inner_margin(egui::style::Margin::same(2.0))
                .show(ui, |ui: &mut egui::Ui| {
                    let id = ui.make_persistent_id(name);
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id,
                        true,
                    )
                    .show_header(ui, |ui| {
                        render_header(ui, graph, node_id, color, &mut remove)
                    })
                    .body(|ui| render_body(ui, graph, parent_node_id, node_id, depth))
                })
        })
        .inner
        .inner
        .2
        .map(|r| r.inner)
        .unwrap_or_default();

    (events, remove)
}

mod util {
    use std::collections::HashSet;

    use super::render_egui_tree;
    use bevy::prelude::*;
    use bevy_egui::egui;
    use shared::{Graph, GraphEvent, Node, NodeData, NodeId};

    fn coloured_button(text: &str, color: egui::color::Hsva) -> egui::Button {
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

    pub fn dragger_row(
        ui: &mut egui::Ui,
        label: &str,
        value: f32,
        default_value: f32,
    ) -> Option<f32> {
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
        with_reset_button(ui, value, default_value, |ui, value| {
            ui.label("Factor");
            let response = ui.add(egui::widgets::Slider::new(value, 0.0..=1.0));
            ui.end_row();
            response.changed()
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
        translation: Vec3,
        rotation: Quat,
        scale: f32,
    ) -> (Option<Vec3>, Option<Quat>, Option<f32>) {
        (
            with_label(ui, "Translation", |ui| vec3(ui, translation, Vec3::ZERO)),
            with_label(ui, "Rotation", |ui| angle(ui, rotation, Quat::IDENTITY)),
            with_label(ui, "Scale", |ui| dragger(ui, scale, 1.0)),
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
        let (translation, rotation, scale) =
            render_transform(ui, node.translation, node.rotation, node.scale);
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

    pub fn render_add_button(
        ui: &mut egui::Ui,
        label: &str,
        include_primitives: bool,
        color: egui::color::Hsva,
    ) -> Option<NodeData> {
        let response = ui.add_sized(
            egui::Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
            coloured_button(label, color),
        );
        render_add_dropdown(ui, response, include_primitives)
    }

    pub fn render_removable_trees(
        ui: &mut egui::Ui,
        graph: &Graph,
        parent_id: NodeId,
        children: &[NodeId],
        depth: usize,
    ) -> impl Iterator<Item = GraphEvent> {
        let depth = depth + 1;
        let mut events = vec![];

        let mut to_remove = HashSet::new();
        for child_id in children {
            let (mut child_events, remove) =
                render_egui_tree(ui, graph, Some(parent_id), *child_id, depth);
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

        let new_child = render_add_button(ui, "Add", true, depth_to_color(depth));
        if let Some(node_data) = new_child {
            events.push(GraphEvent::AddChild(parent_id, None, node_data));
        }

        events.into_iter()
    }

    pub fn render_removable_tree_opt(
        ui: &mut egui::Ui,
        graph: &Graph,
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
                    render_egui_tree(ui, graph, Some(parent_id), child_id, depth);
                events.append(&mut child_events);

                if remove {
                    events.push(GraphEvent::RemoveChild(parent_id, child_id));
                }
            }
            None => {
                let new_child = render_add_button(ui, "Add", true, depth_to_color(depth));
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

    pub fn depth_to_color(depth: usize) -> egui::color::Hsva {
        egui::color::Hsva::new(((depth as f32 / 10.0) * 2.7) % 1.0, 0.6, 0.7, 1.0)
    }
}
