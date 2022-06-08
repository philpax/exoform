use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use super::OccupiedScreenSpace;
use shared::{
    Cylinder, Graph, GraphEvent, Intersect, NodeData, NodeId, Sphere, Subtract, Torus, Union,
};

mod util;

#[derive(Default)]
pub struct SelectedNode(Option<NodeId>);

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedNode>()
            .add_system(sdf_code_editor);
    }
}

fn sdf_code_editor(
    mut egui_context: ResMut<EguiContext>,
    mut graph: ResMut<Graph>,
    mut selected_node: ResMut<SelectedNode>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = egui_context.ctx_mut();
    let mut events = vec![];

    if let Some(selected_node_id) = selected_node.0 {
        if graph.get(selected_node_id).is_none() {
            selected_node.0 = None;
        }
    }

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
            egui::ScrollArea::vertical().show(ui, |ui| {
                events.append(
                    &mut render_egui_tree(
                        ui,
                        &graph,
                        &mut selected_node,
                        None,
                        graph.root_node_id,
                        0,
                    )
                    .0,
                );
            });
        })
        .response
        .rect
        .width();

    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .default_width(400.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(selected_node_id) = selected_node.0 {
                    events.append(&mut render_selected_node(ui, &graph, selected_node_id));
                }
            });
        })
        .response
        .rect
        .width();

    graph.apply_events(&events);
}

fn render_egui_tree(
    ui: &mut egui::Ui,
    graph: &Graph,
    selected_node: &mut SelectedNode,
    parent_node_id: Option<NodeId>,
    node_id: NodeId,
    depth: usize,
) -> (Vec<GraphEvent>, bool) {
    let name = graph.get(node_id).unwrap().data.name().to_owned();

    let mut events = vec![];

    let mut remove = false;
    ui.push_id(node_id, |ui| {
        let id = ui.make_persistent_id(name);
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                events.extend(render_header(
                    ui,
                    graph,
                    selected_node,
                    &mut remove,
                    parent_node_id,
                    node_id,
                    depth,
                ))
            })
            .body(|ui| events.extend(render_body(ui, graph, selected_node, node_id, depth)))
    });

    (events, remove)
}

fn render_header(
    ui: &mut egui::Ui,
    graph: &Graph,
    selected_node: &mut SelectedNode,
    remove: &mut bool,
    parent_node_id: Option<NodeId>,
    node_id: NodeId,
    depth: usize,
) -> Vec<GraphEvent> {
    let mut events = vec![];

    let is_selected = selected_node.0 == Some(node_id);
    let name = graph.get(node_id).unwrap().data.name();

    let color = util::depth_to_color(depth, !is_selected);

    if ui
        .add(util::coloured_button("❌", egui::Color32::LIGHT_RED.into()))
        .clicked()
    {
        *remove = true;
    }

    if let Some(parent_node_id) = parent_node_id {
        if let Some(node_data) =
            util::render_add_parent_button(ui, util::depth_to_color(depth - 1, true))
        {
            events.push(GraphEvent::AddNewParent(parent_node_id, node_id, node_data));
        }
    }

    let interact_size = ui.spacing().interact_size;
    let mut node_text = egui::RichText::new(name)
        .color(color)
        .family(egui::FontFamily::Monospace);
    if is_selected {
        node_text = node_text.color(egui::Color32::WHITE);
    }
    let mut node_button = egui::widgets::Button::new(node_text);
    if is_selected {
        node_button = node_button.fill(color);
    }
    if ui
        .add_sized(
            egui::Vec2::new(ui.available_width(), interact_size.y),
            node_button,
        )
        .clicked()
    {
        selected_node.0 = match selected_node.0 {
            Some(selected_node_id) if selected_node_id == node_id => None,
            _ => Some(node_id),
        };
    }

    events
}

fn render_body(
    ui: &mut egui::Ui,
    graph: &Graph,
    selected_node: &mut SelectedNode,
    node_id: NodeId,
    depth: usize,
) -> Vec<GraphEvent> {
    let mut events = vec![];

    let node = graph.get(node_id).unwrap();
    match &node.data {
        NodeData::Sphere(_) => {}
        NodeData::Cylinder(_) => {}
        NodeData::Torus(_) => {}

        NodeData::Union(Union { children, .. }) => {
            events.extend(util::render_removable_trees(
                ui,
                graph,
                selected_node,
                node_id,
                children,
                depth,
            ));
        }
        NodeData::Intersect(Intersect { children, .. }) => {
            events.extend(util::render_removable_tree_opt(
                ui,
                graph,
                selected_node,
                node_id,
                children.0,
                0,
                depth,
            ));
            events.extend(util::render_removable_tree_opt(
                ui,
                graph,
                selected_node,
                node_id,
                children.1,
                1,
                depth,
            ));
        }
        NodeData::Subtract(Subtract { children, .. }) => {
            events.extend(util::render_removable_trees(
                ui,
                graph,
                selected_node,
                node_id,
                children,
                depth,
            ));
        }
    }

    events
}

fn render_selected_node(ui: &mut egui::Ui, graph: &Graph, node_id: NodeId) -> Vec<GraphEvent> {
    let mut events = vec![];
    let node = graph.get(node_id).unwrap();

    ui.label(egui::RichText::new(node.data.name()).heading());

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
        }
        NodeData::Intersect(Intersect { factor, children }) => {
            let default = Intersect::default();
            let new_factor = util::factor_grid(ui, &mut events, node, *factor, default.factor);
            if let Some(factor) = new_factor {
                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::Intersect(Intersect {
                        factor,
                        children: children.clone(),
                    }),
                ))
            }
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
        }
    }

    events
}
