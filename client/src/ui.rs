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
                            ui.label("Position");
                            vec3(ui, position);
                            ui.end_row();

                            ui.label("Radius");
                            ui.add(dragger(radius));
                            ui.end_row();
                        });
                    }
                    Node::Cylinder {
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
                        factor_slider(ui, factor);
                        render_removable_tree(ui, lhs, 0, depth);
                        render_removable_tree(ui, rhs, 1, depth);
                    }
                    Node::Subtract(factor, (lhs, rhs)) => {
                        factor_slider(ui, factor);
                        render_removable_tree(ui, lhs, 0, depth);
                        render_removable_tree(ui, rhs, 1, depth);
                    }
                    Node::Rgb(r, g, b, child) => {
                        grid(ui, |ui| {
                            ui.label("Colour");
                            let mut rgb = [*r, *g, *b];
                            egui::widgets::color_picker::color_edit_button_rgb(ui, &mut rgb);
                            [*r, *g, *b] = rgb;
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
