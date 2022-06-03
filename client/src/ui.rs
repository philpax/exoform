use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use super::{Graph, OccupiedScreenSpace};
use shared::{Node, NodeData};

pub fn sdf_code_editor(
    mut egui_context: ResMut<EguiContext>,
    mut graph: ResMut<Graph>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = egui_context.ctx_mut();
    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .default_width(400.0)
        .show(ctx, |ui| {
            render_egui_tree(ui, &mut graph.0, 0, 0);
        })
        .response
        .rect
        .width();
}

pub fn render_egui_tree(ui: &mut egui::Ui, node: &mut Node, index: usize, depth: usize) -> bool {
    let color = util::depth_to_color(depth);

    let mut should_remove = false;
    let mut header_renderer = |ui: &mut egui::Ui, node: &mut Node| {
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
            should_remove = true;
        }
    };
    let body_renderer = |ui: &mut egui::Ui, node: &mut Node| {
        if let Some(mut new_node) = util::render_add_parent_button(ui, color) {
            match &mut new_node.data {
                NodeData::Union(_, nodes) => {
                    nodes.push(node.clone());
                }
                NodeData::Intersect(_, (lhs, _)) => *lhs = Some(Box::new(node.clone())),
                NodeData::Subtract(_, nodes) => {
                    nodes.push(node.clone());
                }
                NodeData::Rgb(_, _, _, child_node) => *child_node = Some(Box::new(node.clone())),
                NodeData::Translate(_, child_node) => *child_node = Some(Box::new(node.clone())),
                NodeData::Rotate(_, child_node) => *child_node = Some(Box::new(node.clone())),
                NodeData::Scale(_, child_node) => *child_node = Some(Box::new(node.clone())),
                _ => unreachable!(),
            }
            *node = new_node;
        }

        let default_for_this_node = shared::NODE_DEFAULTS
            .iter()
            .find(|n| std::mem::discriminant(*n) == std::mem::discriminant(&node.data))
            .unwrap();

        match &mut node.data {
            NodeData::Sphere { radius } => {
                let default = match default_for_this_node.clone() {
                    NodeData::Sphere { radius } => radius,
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    util::render_transform(
                        ui,
                        &mut node.translation,
                        &mut node.rotation,
                        &mut node.scale,
                    );

                    ui.label("Radius");
                    util::dragger(ui, radius, default);
                    ui.end_row();
                });
            }
            NodeData::Cylinder {
                cylinder_radius,
                half_height,
                rounding_radius,
            } => {
                let default = match default_for_this_node.clone() {
                    NodeData::Cylinder {
                        cylinder_radius,
                        half_height,
                        rounding_radius,
                    } => (cylinder_radius, half_height, rounding_radius),
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    util::render_transform(
                        ui,
                        &mut node.translation,
                        &mut node.rotation,
                        &mut node.scale,
                    );

                    ui.label("Cylinder radius");
                    util::dragger(ui, cylinder_radius, default.0);
                    ui.end_row();

                    ui.label("Half height");
                    util::dragger(ui, half_height, default.1);
                    ui.end_row();

                    ui.label("Rounding radius");
                    util::dragger(ui, rounding_radius, default.2);
                    ui.end_row();
                });
            }
            NodeData::Torus { big_r, small_r } => {
                let default = match default_for_this_node.clone() {
                    NodeData::Torus { big_r, small_r } => (big_r, small_r),
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    util::render_transform(
                        ui,
                        &mut node.translation,
                        &mut node.rotation,
                        &mut node.scale,
                    );

                    ui.label("Big radius");
                    util::dragger(ui, big_r, default.0);
                    ui.end_row();

                    ui.label("Small radius");
                    util::dragger(ui, small_r, default.1);
                    ui.end_row();
                });
            }

            NodeData::Union(factor, children) => {
                let default = match default_for_this_node {
                    NodeData::Union(factor, ..) => *factor,
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    util::render_transform(
                        ui,
                        &mut node.translation,
                        &mut node.rotation,
                        &mut node.scale,
                    );
                    util::factor_slider(ui, factor, default);
                });

                let mut to_remove = vec![];
                for (index, child) in children.iter_mut().enumerate() {
                    if render_egui_tree(ui, child, index, depth + 1) {
                        to_remove.push(index);
                    }
                }
                to_remove.sort_unstable();
                to_remove.reverse();
                for r in to_remove {
                    children.remove(r);
                }

                if let Some(new) = util::render_add_button(ui, util::depth_to_color(depth + 1)) {
                    children.push(new);
                }
            }
            NodeData::Intersect(factor, (lhs, rhs)) => {
                let default = match default_for_this_node {
                    NodeData::Intersect(factor, ..) => *factor,
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    util::render_transform(
                        ui,
                        &mut node.translation,
                        &mut node.rotation,
                        &mut node.scale,
                    );
                    util::factor_slider(ui, factor, default);
                });
                util::render_removable_tree(ui, lhs, 0, depth);
                util::render_removable_tree(ui, rhs, 1, depth);
            }
            NodeData::Subtract(factor, children) => {
                let default = match default_for_this_node {
                    NodeData::Subtract(factor, ..) => *factor,
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    util::render_transform(
                        ui,
                        &mut node.translation,
                        &mut node.rotation,
                        &mut node.scale,
                    );
                    util::factor_slider(ui, factor, default);
                });

                let mut to_remove = vec![];
                for (index, child) in children.iter_mut().enumerate() {
                    if render_egui_tree(ui, child, index, depth + 1) {
                        to_remove.push(index);
                    }
                }
                to_remove.sort_unstable();
                to_remove.reverse();
                for r in to_remove {
                    children.remove(r);
                }

                if let Some(new) = util::render_add_button(ui, util::depth_to_color(depth + 1)) {
                    children.push(new);
                }
            }

            NodeData::Rgb(r, g, b, child) => {
                let default = match default_for_this_node {
                    NodeData::Rgb(r, g, b, _) => (*r, *g, *b),
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    util::render_transform(
                        ui,
                        &mut node.translation,
                        &mut node.rotation,
                        &mut node.scale,
                    );

                    ui.label("Colour");
                    util::colour(ui, (r, g, b), default);
                    ui.end_row();
                });
                util::render_removable_tree(ui, child, 0, depth);
            }

            NodeData::Translate(position, child) => {
                let default = match default_for_this_node {
                    NodeData::Translate(position, _) => *position,
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    ui.label("Position");
                    util::vec3(ui, position, default);
                    ui.end_row();
                });
                util::render_removable_tree(ui, child, 0, depth);
            }
            NodeData::Rotate(rotation, child) => {
                let default = match default_for_this_node {
                    NodeData::Rotate(rot, _) => *rot,
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    ui.label("Rotation (YPR)");
                    util::angle(ui, rotation, default);
                    ui.end_row();
                });
                util::render_removable_tree(ui, child, 0, depth);
            }
            NodeData::Scale(scale, child) => {
                let default = match default_for_this_node {
                    NodeData::Scale(scale, _) => *scale,
                    _ => unreachable!(),
                };
                util::grid(ui, |ui| {
                    ui.label("Scale");
                    util::dragger(ui, scale, default);
                    ui.end_row();
                });
                util::render_removable_tree(ui, child, 0, depth);
            }
        }
    };

    let renderer = |ui: &mut egui::Ui| {
        let name = node.data.name();
        let id = ui.make_persistent_id(name);
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| header_renderer(ui, node))
            .body(|ui| body_renderer(ui, node));
    };

    ui.push_id(index, |ui| {
        egui::Frame::none()
            .stroke(egui::Stroke::new(1.0, color))
            .inner_margin(egui::style::Margin::same(2.0))
            .show(ui, renderer);
    });

    should_remove
}

mod util {
    use super::render_egui_tree;
    use bevy::prelude::*;
    use bevy_egui::egui;
    use shared::Node;

    pub fn with_reset_button(ui: &mut egui::Ui, main: impl FnOnce(&mut egui::Ui)) -> bool {
        let mut reset = false;
        ui.horizontal(|ui| {
            main(ui);
            if ui
                .small_button(egui::RichText::new("⟳").color(egui::Color32::WHITE))
                .clicked()
            {
                reset = true;
            }
        });
        reset
    }

    pub fn grid(ui: &mut egui::Ui, f: impl FnOnce(&mut egui::Ui)) {
        egui::Grid::new("rows")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, f);
    }

    pub fn dragger_with_no_reset(ui: &mut egui::Ui, value: &mut f32) {
        ui.add(
            egui::widgets::DragValue::new(value)
                .fixed_decimals(2)
                .speed(0.01),
        );
    }

    pub fn dragger(ui: &mut egui::Ui, value: &mut f32, default_value: f32) {
        let reset = with_reset_button(ui, |ui| {
            dragger_with_no_reset(ui, value);
        });
        if reset {
            *value = default_value;
        }
    }

    pub fn vec3(ui: &mut egui::Ui, value: &mut Vec3, default_value: Vec3) {
        let reset = with_reset_button(ui, |ui| {
            ui.horizontal(|ui| {
                dragger_with_no_reset(ui, &mut value.x);
                dragger_with_no_reset(ui, &mut value.y);
                dragger_with_no_reset(ui, &mut value.z);
            });
        });
        if reset {
            *value = default_value;
        }
    }

    pub fn factor_slider(ui: &mut egui::Ui, value: &mut f32, default_value: f32) {
        let reset = with_reset_button(ui, |ui| {
            ui.label("Factor");
            ui.add(egui::widgets::Slider::new(value, 0.0..=1.0));
            ui.end_row();
        });
        if reset {
            *value = default_value;
        }
    }

    pub fn angle(ui: &mut egui::Ui, value: &mut Quat, default_value: Quat) {
        let (mut yaw, mut pitch, mut roll) = value.to_euler(glam::EulerRot::YXZ);
        let reset = with_reset_button(ui, |ui| {
            ui.horizontal(|ui| {
                ui.drag_angle(&mut yaw);
                ui.drag_angle(&mut pitch);
                ui.drag_angle(&mut roll);
            });
            *value = glam::Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll);
        });
        if reset {
            *value = default_value;
        }
    }

    pub fn colour(
        ui: &mut egui::Ui,
        colour: (&mut f32, &mut f32, &mut f32),
        default_value: (f32, f32, f32),
    ) {
        let (r, g, b) = colour;
        let reset = with_reset_button(ui, |ui| {
            let mut rgb = [*r, *g, *b];
            egui::widgets::color_picker::color_edit_button_rgb(ui, &mut rgb);
            [*r, *g, *b] = rgb;
        });
        if reset {
            (*r, *g, *b) = default_value;
        }
    }

    pub fn render_transform(
        ui: &mut egui::Ui,
        translation: &mut Vec3,
        rotation: &mut Quat,
        scale: &mut f32,
    ) {
        ui.label("Translation");
        vec3(ui, translation, Vec3::ZERO);
        ui.end_row();

        ui.label("Rotation");
        angle(ui, rotation, Quat::IDENTITY);
        ui.end_row();

        ui.label("Scale");
        dragger(ui, scale, 1.0);
        ui.end_row();
    }

    pub fn render_removable_tree(
        ui: &mut egui::Ui,
        node: &mut Option<Box<Node>>,
        index: usize,
        depth: usize,
    ) {
        match node {
            Some(inside_node) => {
                if render_egui_tree(ui, inside_node, index, depth + 1) {
                    *node = None;
                }
            }
            None => {
                ui.push_id(index, |ui| {
                    if let Some(new_node) = render_add_button(ui, depth_to_color(depth + 1)) {
                        *node = Some(Box::new(new_node));
                    }
                });
            }
        }
    }

    pub fn render_add_dropdown(
        ui: &mut egui::Ui,
        response: egui::Response,
        include_primitives: bool,
    ) -> Option<Node> {
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
            new_node_data.map(Node::default_with_data)
        })
        .inner
    }

    fn coloured_button(text: &str, color: egui::color::Hsva) -> egui::Button {
        egui::widgets::Button::new(egui::RichText::new(text).color(color)).stroke(egui::Stroke {
            width: 2.0,
            color: color.into(),
        })
    }

    pub fn render_add_button(ui: &mut egui::Ui, color: egui::color::Hsva) -> Option<Node> {
        let response = ui.add_sized(
            egui::Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
            coloured_button("Add", color),
        );
        render_add_dropdown(ui, response, true)
    }

    pub fn render_add_parent_button(ui: &mut egui::Ui, color: egui::color::Hsva) -> Option<Node> {
        let response = ui.add_sized(
            egui::Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
            coloured_button("Add Parent", color),
        );
        render_add_dropdown(ui, response, false)
    }

    pub fn depth_to_color(depth: usize) -> egui::color::Hsva {
        egui::color::Hsva::new(((depth as f32 / 10.0) * 2.7) % 1.0, 0.6, 0.7, 1.0)
    }
}
