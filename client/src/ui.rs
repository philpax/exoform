use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use super::{Graph, OccupiedScreenSpace};
use shared::Node;

pub(crate) fn sdf_code_editor(
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

pub(crate) fn render_egui_tree(
    ui: &mut egui::Ui,
    node: &mut Node,
    index: usize,
    depth: usize,
) -> bool {
    let color = depth_to_color(depth);

    fn with_reset_button(ui: &mut egui::Ui, main: impl FnOnce(&mut egui::Ui)) -> bool {
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

    fn grid(ui: &mut egui::Ui, f: impl FnOnce(&mut egui::Ui)) {
        egui::Grid::new("rows")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, f);
    }

    fn dragger_with_no_reset(ui: &mut egui::Ui, value: &mut f32) {
        ui.add(
            egui::widgets::DragValue::new(value)
                .fixed_decimals(2)
                .speed(0.01),
        );
    }

    fn dragger(ui: &mut egui::Ui, value: &mut f32, default_value: f32) {
        let reset = with_reset_button(ui, |ui| {
            dragger_with_no_reset(ui, value);
        });
        if reset {
            *value = default_value;
        }
    }

    fn vec3(ui: &mut egui::Ui, value: &mut Vec3, default_value: Vec3) {
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

    fn factor_slider(ui: &mut egui::Ui, value: &mut f32, default_value: f32) {
        let reset = with_reset_button(ui, |ui| {
            grid(ui, |ui| {
                ui.label("Factor");
                ui.add(egui::widgets::Slider::new(value, 0.0..=1.0));
                ui.end_row();
            });
        });
        if reset {
            *value = default_value;
        }
    }

    fn angle(ui: &mut egui::Ui, value: &mut Quat, default_value: Quat) {
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

    fn colour(
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

    fn render_removable_tree(
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
                    if let Some(new_node) = render_add_button(ui, depth + 1) {
                        *node = Some(Box::new(new_node));
                    }
                });
            }
        }
    }

    let mut should_remove = false;
    let default_for_this_node = shared::NODE_DEFAULTS
        .iter()
        .find(|n| std::mem::discriminant(*n) == std::mem::discriminant(&*node))
        .unwrap();
    ui.push_id(index, |ui| {
        egui::Frame::none()
            .stroke(egui::Stroke::new(1.0, color))
            .inner_margin(egui::style::Margin::same(2.0))
            .show(ui, |ui| {
                let name = node.name();
                let id = ui.make_persistent_id(name);
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    true,
                )
                .show_header(ui, |ui| {
                    ui.label(
                        egui::RichText::new(name)
                            .color(color)
                            .text_style(egui::TextStyle::Monospace),
                    );

                    if ui
                        .small_button(egui::RichText::new("X").color(egui::Color32::LIGHT_RED))
                        .clicked()
                    {
                        should_remove = true;
                    }
                })
                .body(|ui| match node {
                    Node::Sphere { position, radius } => {
                        grid(ui, |ui| {
                            let default = match default_for_this_node.clone() {
                                Node::Sphere { position, radius } => (position, radius),
                                _ => unreachable!(),
                            };
                            ui.label("Position");
                            vec3(ui, position, default.0);
                            ui.end_row();

                            ui.label("Radius");
                            dragger(ui, radius, default.1);
                            ui.end_row();
                        });
                    }
                    Node::Cylinder {
                        cylinder_radius,
                        half_height,
                        rounding_radius,
                    } => {
                        let default = match default_for_this_node.clone() {
                            Node::Cylinder {
                                cylinder_radius,
                                half_height,
                                rounding_radius,
                            } => (cylinder_radius, half_height, rounding_radius),
                            _ => unreachable!(),
                        };
                        grid(ui, |ui| {
                            ui.label("Cylinder radius");
                            dragger(ui, cylinder_radius, default.0);
                            ui.end_row();

                            ui.label("Half height");
                            dragger(ui, half_height, default.1);
                            ui.end_row();

                            ui.label("Rounding radius");
                            dragger(ui, rounding_radius, default.2);
                            ui.end_row();
                        });
                    }
                    Node::Torus { big_r, small_r } => {
                        let default = match default_for_this_node.clone() {
                            Node::Torus { big_r, small_r } => (big_r, small_r),
                            _ => unreachable!(),
                        };
                        grid(ui, |ui| {
                            ui.label("Big radius");
                            dragger(ui, big_r, default.0);
                            ui.end_row();

                            ui.label("Small radius");
                            dragger(ui, small_r, default.1);
                            ui.end_row();
                        });
                    }

                    Node::Union(factor, children) => {
                        let default = match default_for_this_node {
                            Node::Union(factor, ..) => *factor,
                            _ => unreachable!(),
                        };
                        factor_slider(ui, factor, default);
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

                        if let Some(new_node) = render_add_button(ui, depth + 1) {
                            children.push(new_node);
                        }
                    }
                    Node::Intersect(factor, (lhs, rhs)) => {
                        let default = match default_for_this_node {
                            Node::Intersect(factor, ..) => *factor,
                            _ => unreachable!(),
                        };
                        factor_slider(ui, factor, default);
                        render_removable_tree(ui, lhs, 0, depth);
                        render_removable_tree(ui, rhs, 1, depth);
                    }
                    Node::Subtract(factor, (lhs, rhs)) => {
                        let default = match default_for_this_node {
                            Node::Subtract(factor, ..) => *factor,
                            _ => unreachable!(),
                        };
                        factor_slider(ui, factor, default);
                        render_removable_tree(ui, lhs, 0, depth);
                        render_removable_tree(ui, rhs, 1, depth);
                    }

                    Node::Rgb(r, g, b, child) => {
                        let default = match default_for_this_node {
                            Node::Rgb(r, g, b, _) => (*r, *g, *b),
                            _ => unreachable!(),
                        };
                        grid(ui, |ui| {
                            ui.label("Colour");
                            colour(ui, (r, g, b), default);
                            ui.end_row();
                        });
                        render_removable_tree(ui, child, 0, depth);
                    }

                    Node::Translate(position, child) => {
                        let default = match default_for_this_node {
                            Node::Translate(position, _) => *position,
                            _ => unreachable!(),
                        };
                        grid(ui, |ui| {
                            ui.label("Position");
                            vec3(ui, position, default);
                            ui.end_row();
                        });
                        render_removable_tree(ui, child, 0, depth);
                    }
                    Node::Rotate(rotation, child) => {
                        let default = match default_for_this_node {
                            Node::Rotate(rot, _) => *rot,
                            _ => unreachable!(),
                        };
                        grid(ui, |ui| {
                            ui.label("Rotation (YPR)");
                            angle(ui, rotation, default);
                            ui.end_row();
                        });
                        render_removable_tree(ui, child, 0, depth);
                    }
                    Node::Scale(scale, child) => {
                        let default = match default_for_this_node {
                            Node::Scale(scale, _) => *scale,
                            _ => unreachable!(),
                        };
                        grid(ui, |ui| {
                            ui.label("Scale");
                            dragger(ui, scale, default);
                            ui.end_row();
                        });
                        render_removable_tree(ui, child, 0, depth);
                    }
                });
            });
    });

    should_remove
}

pub(crate) fn render_add_button(ui: &mut egui::Ui, depth: usize) -> Option<Node> {
    let color = depth_to_color(depth);
    let response = ui.add_sized(
        egui::Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
        egui::widgets::Button::new(egui::RichText::new("Add").color(color)).stroke(egui::Stroke {
            width: 2.0,
            color: color.into(),
        }),
    );
    let popup_id = ui.make_persistent_id("add_menu");
    if response.clicked() {
        ui.memory().toggle_popup(popup_id);
    }
    let mut new_node = None;
    egui::popup_below_widget(ui, popup_id, &response, |ui| {
        for default in shared::NODE_DEFAULTS.iter() {
            let category_color = match default.category() {
                shared::NodeCategory::Primitive => egui::Color32::from_rgb(78, 205, 196),
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
                new_node = Some(default.clone());
            }
        }
    });
    new_node
}

pub(crate) fn depth_to_color(depth: usize) -> egui::color::Hsva {
    egui::color::Hsva::new(((depth as f32 / 10.0) * 2.7) % 1.0, 0.6, 0.7, 1.0)
}
