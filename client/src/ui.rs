use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{NetworkState, RenderParameters};

use super::OccupiedScreenSpace;
use shared::{Node, *};

mod util;

#[derive(Default, PartialEq)]
enum SelectedNode {
    #[default]
    Uninitialized,
    Initialized(Option<NodeId>),
}
impl SelectedNode {
    fn is_selected(&self, node_id: NodeId) -> bool {
        match self {
            Self::Uninitialized => false,
            Self::Initialized(inside_node_id) => *inside_node_id == Some(node_id),
        }
    }

    fn select(&mut self, node_id: NodeId) {
        *self = Self::Initialized(match *self {
            Self::Initialized(Some(selected_node_id)) if selected_node_id == node_id => None,
            _ => Some(node_id),
        });
    }
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedNode>()
            .add_system(sdf_code_editor);
    }
}

fn sdf_code_editor(
    mut egui_context: ResMut<EguiContext>,
    mut selected_node: ResMut<SelectedNode>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut render_parameters: ResMut<RenderParameters>,
    mut network_state: ResMut<NetworkState>,
    graph: Res<Graph>,
) {
    let ctx = egui_context.ctx_mut();
    let mut commands = vec![];

    match *selected_node {
        SelectedNode::Uninitialized => {
            if let Some(root_node_id) = graph.root_node_id() {
                selected_node.select(root_node_id);
            }
        }
        SelectedNode::Initialized(Some(selected_node_id)) => {
            // clear the selected node if the node no longer exists in the graph
            if graph.get(selected_node_id).is_none() {
                *selected_node = SelectedNode::Initialized(None);
            }
        }
        _ => {}
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
            if let Some(root_node_id) = graph.root_node_id() {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    commands.append(&mut render_egui_tree(
                        ui,
                        &graph,
                        &mut selected_node,
                        None,
                        root_node_id,
                        0,
                    ));
                });
            }
        })
        .response
        .rect
        .width();

    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .default_width(400.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Parameters");
                ui.checkbox(&mut render_parameters.wireframe, "Wireframe");
            });
        })
        .response
        .rect
        .width();

    network_state.send(&commands);
}

fn render_egui_tree(
    ui: &mut egui::Ui,
    graph: &Graph,
    selected_node: &mut SelectedNode,
    parent_node_id: Option<NodeId>,
    node_id: NodeId,
    depth: usize,
) -> Vec<GraphCommand> {
    let node = graph.get(node_id).unwrap();

    let mut commands = vec![];
    ui.push_id(node_id, |ui| {
        let id = ui.make_persistent_id(node.data.name());

        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                commands.extend(render_header(
                    ui,
                    graph,
                    selected_node,
                    parent_node_id,
                    node_id,
                    depth,
                ))
            })
            .body(|ui| {
                egui::CollapsingHeader::new("Parameters")
                    .default_open(true)
                    .show(ui, |ui| {
                        commands.extend(render_selected_node(ui, node, depth));
                    });
                if node.data.can_have_children() {
                    commands.extend(render_children(ui, graph, selected_node, node, depth));
                }
            });
    });

    commands
}

fn render_header(
    ui: &mut egui::Ui,
    graph: &Graph,
    selected_node: &mut SelectedNode,
    parent_node_id: Option<NodeId>,
    node_id: NodeId,
    depth: usize,
) -> Vec<GraphCommand> {
    let mut commands = vec![];

    let interact_size = ui.spacing().interact_size;
    let is_selected = selected_node.is_selected(node_id);
    let name = graph.get(node_id).unwrap().data.name();
    let (bg_colour, fg_colour) = (
        util::depth_to_colour(depth, is_selected),
        egui::Color32::WHITE,
    );

    let response = ui.add_sized(
        egui::Vec2::new(ui.available_width(), interact_size.y),
        egui::Button::new(
            egui::RichText::new(name)
                .color(fg_colour)
                .family(egui::FontFamily::Monospace),
        )
        .fill(bg_colour)
        .sense(egui::Sense::click()),
    );
    if response.clicked_by(egui::PointerButton::Primary) {
        selected_node.select(node_id);
    }
    if let Some(parent_node_id) = parent_node_id {
        response.context_menu(|ui| {
            ui.menu_button("Add Parent", |ui| {
                if let Some(node_data) = util::render_add_buttons(ui, false) {
                    commands.push(GraphCommand::AddNewParent(
                        parent_node_id,
                        node_id,
                        node_data,
                    ));
                    ui.close_menu();
                }
            });

            if ui.button("Delete").clicked() {
                commands.push(GraphCommand::RemoveChild(parent_node_id, node_id));
                ui.close_menu();
            }
        });
    }

    commands
}

fn render_children(
    ui: &mut egui::Ui,
    graph: &Graph,
    selected_node: &mut SelectedNode,
    parent: &Node,
    depth: usize,
) -> Vec<GraphCommand> {
    let depth = depth + 1;
    let mut commands: Vec<_> = parent
        .children
        .iter()
        .enumerate()
        .flat_map(|(idx, child_id)| match *child_id {
            Some(child_id) => {
                render_egui_tree(ui, graph, selected_node, Some(parent.id), child_id, depth)
            }
            None => util::render_add_button(ui, depth, parent.id, Some(idx))
                .into_iter()
                .collect(),
        })
        .collect();

    if parent.data.can_have_children() {
        let new_child = util::render_add_button_max_width(ui, util::depth_to_colour(depth, false));
        if let Some(node_data) = new_child {
            commands.push(GraphCommand::AddChild(parent.id, None, node_data));
        }
    }

    commands
}

fn render_selected_node(ui: &mut egui::Ui, node: &Node, depth: usize) -> Option<GraphCommand> {
    util::grid(ui, |ui| {
        NodeDiff {
            rgb: util::with_label(ui, "Colour", |ui| {
                let depth_colour = util::depth_to_colour(depth, false);

                util::with_reset_button(ui, node.rgb, Node::DEFAULT_COLOUR, |ui, (r, g, b)| {
                    let mut rgb = [*r, *g, *b];
                    let widget_changed =
                        egui::widgets::color_picker::color_edit_button_rgb(ui, &mut rgb).changed();
                    let button_clicked = ui
                        .add(util::coloured_button("Depth", depth_colour))
                        .clicked();
                    if button_clicked {
                        rgb = depth_colour.to_rgb();
                    }
                    [*r, *g, *b] = rgb;
                    widget_changed || button_clicked
                })
            }),
            transform: util::render_transform(ui, &node.transform),
            data: render_selected_node_data(ui, node),
            children: None,
        }
        .into_option()
        .map(|d| GraphCommand::ApplyDiff(node.id, d))
    })
}

fn render_selected_node_data(ui: &mut egui::Ui, node: &Node) -> Option<NodeDataDiff> {
    use util::dragger_row as row;
    macro_rules! apply_diff {
        ($($diff:tt)*) => {{
            let diff = $($diff)*;
            diff.into_option().map(|d| d.into())
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
