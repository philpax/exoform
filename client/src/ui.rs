use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use super::OccupiedScreenSpace;
use shared::{
    Cylinder, Graph, GraphEvent, Intersect, NodeData, NodeId, Sphere, Subtract, Torus, Union,
};

mod util;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(sdf_code_editor);
    }
}

fn sdf_code_editor(
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
