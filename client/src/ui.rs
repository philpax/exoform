use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use super::OccupiedScreenSpace;
use shared::{
    BiconvexLens, Box, Capsule, Cone, Cylinder, Graph, GraphEvent, Intersect, NodeData, NodeId,
    Plane, Sphere, Subtract, TaperedCapsule, Torus, TorusSector, Union,
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
    let (bg_colour, fg_colour) = (
        util::depth_to_color(depth, is_selected),
        egui::Color32::WHITE,
    );
    if ui
        .add_sized(
            egui::Vec2::new(ui.available_width(), interact_size.y),
            egui::Button::new(
                egui::RichText::new(name)
                    .color(fg_colour)
                    .family(egui::FontFamily::Monospace),
            )
            .fill(bg_colour)
            .sense(egui::Sense::click()),
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
        NodeData::Plane(_) => {}
        NodeData::Capsule(_) => {}
        NodeData::TaperedCapsule(_) => {}
        NodeData::Cone(_) => {}
        NodeData::Box(_) => {}
        NodeData::TorusSector(_) => {}
        NodeData::BiconvexLens(_) => {}

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
    macro_rules! return_if_unchanged {
        [$($value:ident),*] => {
            if [$($value.is_none()),*].into_iter().all(|x| x) {
                return;
            }
        }
    }

    let mut events = vec![];
    let node = graph.get(node_id).unwrap();

    ui.label(egui::RichText::new(node.data.name()).heading());

    use util::dragger_row as row;

    match &node.data {
        NodeData::Sphere(Sphere { radius }) => {
            let default = Sphere::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_radius = row(ui, "Radius", *radius, default.radius);
                if let Some(radius) = new_radius {
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

                let new_cylinder_radius = row(
                    ui,
                    "Cylinder radius",
                    *cylinder_radius,
                    default.cylinder_radius,
                );
                let new_half_height = row(ui, "Half height", *half_height, default.half_height);
                let new_rounding_radius = row(
                    ui,
                    "Rounding radius",
                    *rounding_radius,
                    default.rounding_radius,
                );

                return_if_unchanged![new_cylinder_radius, new_half_height, new_rounding_radius];

                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::Cylinder(Cylinder {
                        cylinder_radius: new_cylinder_radius.unwrap_or(*cylinder_radius),
                        half_height: new_half_height.unwrap_or(*half_height),
                        rounding_radius: new_rounding_radius.unwrap_or(*rounding_radius),
                    }),
                ));
            });
        }
        NodeData::Torus(Torus { big_r, small_r }) => {
            let default = Torus::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_big_r = row(ui, "Big radius", *big_r, default.big_r);
                let new_small_r = row(ui, "Small radius", *small_r, default.small_r);

                return_if_unchanged![new_big_r, new_small_r];

                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::Torus(Torus {
                        big_r: new_big_r.unwrap_or(*big_r),
                        small_r: new_small_r.unwrap_or(*small_r),
                    }),
                ));
            });
        }
        NodeData::Plane(Plane { .. }) => {
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));
            });
        }
        NodeData::Capsule(Capsule { points, radius }) => {
            let default = Capsule::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_points_0 = util::with_label(ui, "Point 1", |ui| {
                    util::vec3(ui, points[0], default.points[0])
                });
                let new_points_1 = util::with_label(ui, "Point 2", |ui| {
                    util::vec3(ui, points[1], default.points[1])
                });
                let new_radius = row(ui, "Radius", *radius, default.radius);

                return_if_unchanged![new_points_0, new_points_1, new_radius];

                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::Capsule(Capsule {
                        points: [
                            new_points_0.unwrap_or(default.points[0]),
                            new_points_1.unwrap_or(default.points[1]),
                        ],
                        radius: new_radius.unwrap_or(*radius),
                    }),
                ));
            });
        }
        NodeData::TaperedCapsule(TaperedCapsule { points, radii }) => {
            let default = TaperedCapsule::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_points_0 = util::with_label(ui, "Point 1", |ui| {
                    util::vec3(ui, points[0], default.points[0])
                });
                let new_points_1 = util::with_label(ui, "Point 2", |ui| {
                    util::vec3(ui, points[1], default.points[1])
                });
                let new_radius_0 = row(ui, "Radius 1", radii[0], default.radii[0]);
                let new_radius_1 = row(ui, "Radius 2", radii[1], default.radii[1]);

                return_if_unchanged![new_points_0, new_points_1, new_radius_0, new_radius_1];

                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::TaperedCapsule(TaperedCapsule {
                        points: [
                            new_points_0.unwrap_or(default.points[0]),
                            new_points_1.unwrap_or(default.points[1]),
                        ],
                        radii: [
                            new_radius_0.unwrap_or(default.radii[0]),
                            new_radius_1.unwrap_or(default.radii[1]),
                        ],
                    }),
                ));
            });
        }
        NodeData::Cone(Cone { radius, height }) => {
            let default = Cone::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_radius = row(ui, "Radius", *radius, default.radius);
                let new_height = row(ui, "Height", *height, default.height);

                return_if_unchanged![new_radius, new_height];

                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::Cone(Cone {
                        radius: new_radius.unwrap_or(*radius),
                        height: new_height.unwrap_or(*height),
                    }),
                ));
            });
        }
        NodeData::Box(Box {
            half_size,
            rounding_radius,
        }) => {
            let default = Box::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_half_size = util::with_label(ui, "Half size", |ui| {
                    util::vec3(ui, *half_size, default.half_size)
                });
                let new_rounding_radius = row(
                    ui,
                    "Rounding radius",
                    *rounding_radius,
                    default.rounding_radius,
                );

                return_if_unchanged![new_half_size, new_rounding_radius];

                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::Box(Box {
                        half_size: new_half_size.unwrap_or(*half_size),
                        rounding_radius: new_rounding_radius.unwrap_or(*rounding_radius),
                    }),
                ));
            });
        }
        NodeData::TorusSector(TorusSector {
            big_r,
            small_r,
            angle,
        }) => {
            let default = TorusSector::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_big_r = row(ui, "Big radius", *big_r, default.big_r);
                let new_small_r = row(ui, "Small radius", *small_r, default.small_r);
                let new_angle = util::with_label(ui, "Angle", |ui| {
                    util::with_reset_button(ui, *angle, default.angle, |ui, value| {
                        let changed = ui.drag_angle(value).changed();
                        *value %= std::f32::consts::TAU;
                        changed
                    })
                });

                return_if_unchanged![new_big_r, new_small_r, new_angle];

                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::TorusSector(TorusSector {
                        big_r: new_big_r.unwrap_or(*big_r),
                        small_r: new_small_r.unwrap_or(*small_r),
                        angle: new_angle.unwrap_or(*angle),
                    }),
                ));
            });
        }
        NodeData::BiconvexLens(BiconvexLens {
            lower_sagitta,
            upper_sagitta,
            chord,
        }) => {
            let default = BiconvexLens::default();
            util::grid(ui, |ui| {
                events.extend(util::render_node_prelude_with_events(ui, node));

                let new_lower_sagitta =
                    row(ui, "Lower sagitta", *lower_sagitta, default.lower_sagitta);
                let new_upper_sagitta =
                    row(ui, "Upper sagitta", *upper_sagitta, default.upper_sagitta);
                let new_chord = row(ui, "Chord", *chord, default.chord);

                return_if_unchanged![new_lower_sagitta, new_upper_sagitta, new_chord];

                events.push(GraphEvent::ReplaceData(
                    node_id,
                    NodeData::BiconvexLens(BiconvexLens {
                        lower_sagitta: new_lower_sagitta.unwrap_or(*lower_sagitta),
                        upper_sagitta: new_upper_sagitta.unwrap_or(*upper_sagitta),
                        chord: new_chord.unwrap_or(*chord),
                    }),
                ));
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
