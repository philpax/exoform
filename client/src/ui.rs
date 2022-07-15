use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use super::OccupiedScreenSpace;
use shared::*;

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
                events.append(&mut util::render_egui_tree(
                    ui,
                    &graph,
                    &mut selected_node,
                    None,
                    graph.root_node_id,
                    0,
                ));
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

fn render_selected_node(ui: &mut egui::Ui, graph: &Graph, node_id: NodeId) -> Vec<GraphEvent> {
    let mut events = vec![];
    let node = graph.get(node_id).unwrap();

    ui.label(egui::RichText::new(node.data.name()).heading());
    util::grid(ui, |ui| {
        events.extend(util::render_node_prelude_with_events(ui, node));
        if let Some(event) = render_selected_node_data(ui, node) {
            events.push(event);
        }
    });

    events
}

fn render_selected_node_data(ui: &mut egui::Ui, node: &shared::Node) -> Option<GraphEvent> {
    use util::dragger_row as row;
    macro_rules! apply_diff {
        ($($diff:tt)*) => {{
            let diff = $($diff)*;
            if diff.has_changes() {
                Some(GraphEvent::ApplyDiff(node.id, diff.into()))
            } else {
                None
            }
        }};
    }

    match &node.data {
        NodeData::Sphere(Sphere { radius }) => {
            let default = Sphere::default();
            apply_diff!(SphereDiff {
                radius: row(ui, "Radius", *radius, default.radius),
            })
        }
        NodeData::Cylinder(Cylinder {
            cylinder_radius,
            half_height,
            rounding_radius,
        }) => {
            let default = Cylinder::default();
            apply_diff!(CylinderDiff {
                cylinder_radius: row(
                    ui,
                    "Cylinder radius",
                    *cylinder_radius,
                    default.cylinder_radius,
                ),
                half_height: row(ui, "Half height", *half_height, default.half_height),
                rounding_radius: row(
                    ui,
                    "Rounding radius",
                    *rounding_radius,
                    default.rounding_radius,
                ),
            })
        }
        NodeData::Torus(Torus { big_r, small_r }) => {
            let default = Torus::default();
            apply_diff!(TorusDiff {
                big_r: row(ui, "Big radius", *big_r, default.big_r),
                small_r: row(ui, "Small radius", *small_r, default.small_r),
            })
        }
        NodeData::Plane(Plane { .. }) => None,
        NodeData::Capsule(Capsule {
            point_1,
            point_2,
            radius,
        }) => {
            let default = Capsule::default();
            apply_diff!(CapsuleDiff {
                point_1: util::with_label(ui, "Point 1", |ui| {
                    util::vec3(ui, *point_1, default.point_1)
                }),
                point_2: util::with_label(ui, "Point 2", |ui| {
                    util::vec3(ui, *point_2, default.point_2)
                }),
                radius: row(ui, "Radius", *radius, default.radius),
            })
        }
        NodeData::TaperedCapsule(TaperedCapsule {
            point_1,
            point_2,
            radius_1,
            radius_2,
        }) => {
            let default = TaperedCapsule::default();
            apply_diff!(TaperedCapsuleDiff {
                point_1: util::with_label(ui, "Point 1", |ui| {
                    util::vec3(ui, *point_1, default.point_1)
                }),
                point_2: util::with_label(ui, "Point 2", |ui| {
                    util::vec3(ui, *point_2, default.point_2)
                }),
                radius_1: row(ui, "Radius 1", *radius_1, default.radius_1),
                radius_2: row(ui, "Radius 2", *radius_2, default.radius_2),
            })
        }
        NodeData::Cone(Cone { radius, height }) => {
            let default = Cone::default();
            apply_diff!(ConeDiff {
                radius: row(ui, "Radius", *radius, default.radius),
                height: row(ui, "Height", *height, default.height),
            })
        }
        NodeData::Box(Box {
            half_size,
            rounding_radius,
        }) => {
            let default = Box::default();
            apply_diff!(BoxDiff {
                half_size: util::with_label(ui, "Half size", |ui| {
                    util::vec3(ui, *half_size, default.half_size)
                }),
                rounding_radius: row(
                    ui,
                    "Rounding radius",
                    *rounding_radius,
                    default.rounding_radius,
                ),
            })
        }
        NodeData::TorusSector(TorusSector {
            big_r,
            small_r,
            angle,
        }) => {
            let default = TorusSector::default();
            apply_diff!(TorusSectorDiff {
                big_r: row(ui, "Big radius", *big_r, default.big_r),
                small_r: row(ui, "Small radius", *small_r, default.small_r),
                angle: util::with_label(ui, "Angle", |ui| {
                    util::with_reset_button(ui, *angle, default.angle, |ui, value| {
                        let changed = ui.drag_angle(value).changed();
                        *value %= std::f32::consts::TAU;
                        changed
                    })
                })
            })
        }
        NodeData::BiconvexLens(BiconvexLens {
            lower_sagitta,
            upper_sagitta,
            chord,
        }) => {
            let default = BiconvexLens::default();
            apply_diff!(BiconvexLensDiff {
                lower_sagitta: row(ui, "Lower sagitta", *lower_sagitta, default.lower_sagitta),
                upper_sagitta: row(ui, "Upper sagitta", *upper_sagitta, default.upper_sagitta),
                chord: row(ui, "Chord", *chord, default.chord),
            })
        }

        NodeData::Union(Union { factor }) => {
            let default = Union::default();
            apply_diff!(UnionDiff {
                factor: util::factor_slider(ui, *factor, default.factor)
            })
        }
        NodeData::Intersect(Intersect { factor }) => {
            let default = Intersect::default();
            apply_diff!(IntersectDiff {
                factor: util::factor_slider(ui, *factor, default.factor)
            })
        }
        NodeData::Subtract(Subtract { factor }) => {
            let default = Subtract::default();
            apply_diff!(SubtractDiff {
                factor: util::factor_slider(ui, *factor, default.factor)
            })
        }
    }
}
