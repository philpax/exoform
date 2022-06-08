use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use glam::{Vec3, Quat};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(u32);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeCategory {
    Primitive,
    Operation,
    Metadata,
    Transform,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sphere {
    pub radius: f32,
}
impl Sphere {
    pub const fn new() -> Sphere {
        Sphere { radius: 1.0 }
    }
}
impl Default for Sphere {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cylinder {
    pub cylinder_radius: f32,
    pub half_height: f32,
    pub rounding_radius: f32,
}
impl Cylinder {
    pub const fn new() -> Cylinder {
        Cylinder {
            cylinder_radius: 1.0,
            half_height: 1.0,
            rounding_radius: 0.0,
        }
    }
}
impl Default for Cylinder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Torus {
    pub big_r: f32,
    pub small_r: f32,
}
impl Torus {
    pub const fn new() -> Torus {
        Torus {
            big_r: 1.0,
            small_r: 0.1,
        }
    }
}
impl Default for Torus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Union {
    pub factor: f32,
    pub children: Vec<NodeId>,
}
impl Union {
    pub const fn new() -> Union {
        Union {
            factor: 0.0,
            children: vec![],
        }
    }
}
impl Default for Union {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intersect {
    pub factor: f32,
    pub children: (Option<NodeId>, Option<NodeId>),
}
impl Intersect {
    pub const fn new() -> Intersect {
        Intersect {
            factor: 0.0,
            children: (None, None),
        }
    }
}
impl Default for Intersect {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subtract {
    pub factor: f32,
    pub children: Vec<NodeId>,
}
impl Subtract {
    pub const fn new() -> Subtract {
        Subtract {
            factor: 0.0,
            children: vec![],
        }
    }
}
impl Default for Subtract {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
    pub rgb: (f32, f32, f32),
    pub child: Option<NodeId>,
}
impl Rgb {
    pub const fn new() -> Rgb {
        Rgb {
            rgb: (1.0, 1.0, 1.0),
            child: None,
        }
    }
}
impl Default for Rgb {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Translate {
    pub position: Vec3,
    pub child: Option<NodeId>,
}
impl Translate {
    pub const fn new() -> Translate {
        Translate {
            position: Vec3::ZERO,
            child: None,
        }
    }
}
impl Default for Translate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rotate {
    pub rotation: Quat,
    pub child: Option<NodeId>,
}
impl Rotate {
    pub const fn new() -> Rotate {
        Rotate {
            rotation: Quat::IDENTITY,
            child: None,
        }
    }
}
impl Default for Rotate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub scale: f32,
    pub child: Option<NodeId>,
}
impl Scale {
    pub const fn new() -> Scale {
        Scale {
            scale: 1.0,
            child: None,
        }
    }
}
impl Default for Scale {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeData {
    Sphere(Sphere),
    Cylinder(Cylinder),
    Torus(Torus),

    Union(Union),
    Intersect(Intersect),
    Subtract(Subtract),

    Rgb(Rgb),

    Translate(Translate),
    Rotate(Rotate),
    Scale(Scale),
}
impl NodeData {
    pub const fn name(&self) -> &str {
        match self {
            NodeData::Sphere(_) => "Sphere",
            NodeData::Cylinder(_) => "Cylinder",
            NodeData::Torus(_) => "Torus",

            NodeData::Union(_) => "Union",
            NodeData::Intersect(_) => "Intersect",
            NodeData::Subtract(_) => "Subtract",

            NodeData::Rgb(_) => "Rgb",

            NodeData::Translate(_) => "Translate",
            NodeData::Rotate(_) => "Rotate",
            NodeData::Scale(_) => "Scale",
        }
    }

    pub const fn category(&self) -> NodeCategory {
        match self {
            NodeData::Sphere(_) => NodeCategory::Primitive,
            NodeData::Cylinder(_) => NodeCategory::Primitive,
            NodeData::Torus(_) => NodeCategory::Primitive,

            NodeData::Union(_) => NodeCategory::Operation,
            NodeData::Intersect(_) => NodeCategory::Operation,
            NodeData::Subtract(_) => NodeCategory::Operation,

            NodeData::Rgb(_) => NodeCategory::Metadata,

            NodeData::Translate(_) => NodeCategory::Transform,
            NodeData::Rotate(_) => NodeCategory::Transform,
            NodeData::Scale(_) => NodeCategory::Transform,
        }
    }

    pub fn add_child(&mut self, index: Option<usize>, child_id: NodeId) {
        fn add_to_vec(children: &mut Vec<NodeId>, index: Option<usize>, child_id: NodeId) {
            match index {
                Some(index) => *children.get_mut(index).unwrap() = child_id,
                None => children.push(child_id),
            }
        }

        fn add_to_lhs_rhs(
            children: &mut (Option<NodeId>, Option<NodeId>),
            index: Option<usize>,
            child_id: NodeId,
        ) {
            match index {
                Some(0) => children.0 = Some(child_id),
                Some(1) => children.1 = Some(child_id),
                Some(_) => panic!("out of bounds index"),
                None => match children {
                    (None, _) => children.0 = Some(child_id),
                    (_, None) => children.1 = Some(child_id),
                    (Some(_), Some(_)) => {
                        panic!("tried to add a new child, but both slots were full")
                    }
                },
            }
        }

        fn add_to_one(child: &mut Option<NodeId>, child_id: NodeId) {
            *child = Some(child_id)
        }

        match self {
            NodeData::Sphere(_) => panic!("this node does not support children"),
            NodeData::Cylinder(_) => panic!("this node does not support children"),
            NodeData::Torus(_) => panic!("this node does not support children"),

            NodeData::Union(Union { children, .. }) => add_to_vec(children, index, child_id),
            NodeData::Intersect(Intersect { children, .. }) => {
                add_to_lhs_rhs(children, index, child_id)
            }
            NodeData::Subtract(Subtract { children, .. }) => add_to_vec(children, index, child_id),

            NodeData::Rgb(Rgb { child, .. }) => add_to_one(child, child_id),

            NodeData::Translate(Translate { child, .. }) => add_to_one(child, child_id),
            NodeData::Rotate(Rotate { child, .. }) => add_to_one(child, child_id),
            NodeData::Scale(Scale { child, .. }) => add_to_one(child, child_id),
        }
    }

    fn remove_child(&mut self, child_id: NodeId) {
        fn remove_from_vec(children: &mut Vec<NodeId>, child_id: NodeId) {
            children.retain(|id| *id != child_id);
        }

        fn remove_from_lhs_rhs(children: &mut (Option<NodeId>, Option<NodeId>), child_id: NodeId) {
            if children.0 == Some(child_id) {
                children.0 = None;
            }

            if children.1 == Some(child_id) {
                children.1 = None;
            }
        }

        fn remove_from_one(child: &mut Option<NodeId>) {
            *child = None
        }

        match self {
            NodeData::Sphere(_) => panic!("this node does not support children"),
            NodeData::Cylinder(_) => panic!("this node does not support children"),
            NodeData::Torus(_) => panic!("this node does not support children"),

            NodeData::Union(Union { children, .. }) => remove_from_vec(children, child_id),
            NodeData::Intersect(Intersect { children, .. }) => {
                remove_from_lhs_rhs(children, child_id)
            }
            NodeData::Subtract(Subtract { children, .. }) => remove_from_vec(children, child_id),

            NodeData::Rgb(Rgb { child, .. }) => remove_from_one(child),

            NodeData::Translate(Translate { child, .. }) => remove_from_one(child),
            NodeData::Rotate(Rotate { child, .. }) => remove_from_one(child),
            NodeData::Scale(Scale { child, .. }) => remove_from_one(child),
        }
    }

    fn replace_child(&mut self, old_child_id: NodeId, new_child_id: NodeId) {
        fn replace_in_vec(children: &mut Vec<NodeId>, old_child_id: NodeId, new_child_id: NodeId) {
            for child_id in children {
                if *child_id == old_child_id {
                    *child_id = new_child_id;
                }
            }
        }

        fn replace_in_lhs_rhs(
            children: &mut (Option<NodeId>, Option<NodeId>),
            old_child_id: NodeId,
            new_child_id: NodeId,
        ) {
            if children.0 == Some(old_child_id) {
                children.0 = Some(new_child_id);
            }

            if children.1 == Some(old_child_id) {
                children.1 = Some(new_child_id);
            }
        }

        fn replace_in_one(child: &mut Option<NodeId>, old_child_id: NodeId, new_child_id: NodeId) {
            if *child == Some(old_child_id) {
                *child = Some(new_child_id);
            }
        }

        match self {
            NodeData::Sphere(_) => panic!("this node does not support children"),
            NodeData::Cylinder(_) => panic!("this node does not support children"),
            NodeData::Torus(_) => panic!("this node does not support children"),

            NodeData::Union(Union { children, .. }) => {
                replace_in_vec(children, old_child_id, new_child_id)
            }
            NodeData::Intersect(Intersect { children, .. }) => {
                replace_in_lhs_rhs(children, old_child_id, new_child_id)
            }
            NodeData::Subtract(Subtract { children, .. }) => {
                replace_in_vec(children, old_child_id, new_child_id)
            }

            NodeData::Rgb(Rgb { child, .. }) => replace_in_one(child, old_child_id, new_child_id),

            NodeData::Translate(Translate { child, .. }) => {
                replace_in_one(child, old_child_id, new_child_id)
            }
            NodeData::Rotate(Rotate { child, .. }) => {
                replace_in_one(child, old_child_id, new_child_id)
            }
            NodeData::Scale(Scale { child, .. }) => {
                replace_in_one(child, old_child_id, new_child_id)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: f32,
    pub data: NodeData,
}
impl Node {
    pub const fn new(id: NodeId, data: NodeData) -> Node {
        Node {
            id,
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
            data,
        }
    }
}
impl ToString for Node {
    fn to_string(&self) -> String {
        let mut buf = Vec::new();
        let mut serializer = serde_json::ser::Serializer::with_formatter(
            &mut buf,
            serde_json::ser::PrettyFormatter::with_indent(b" "),
        );
        self.serialize(&mut serializer).unwrap();
        String::from_utf8(buf).unwrap()
    }
}

pub const NODE_DEFAULTS: &[NodeData] = &[
    NodeData::Sphere(Sphere::new()),
    NodeData::Cylinder(Cylinder::new()),
    NodeData::Torus(Torus::new()),
    NodeData::Union(Union::new()),
    NodeData::Intersect(Intersect::new()),
    NodeData::Subtract(Subtract::new()),
    NodeData::Rgb(Rgb::new()),
    NodeData::Translate(Translate::new()),
    NodeData::Rotate(Rotate::new()),
    NodeData::Scale(Scale::new()),
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphEvent {
    AddChild(NodeId, Option<usize>, NodeData),
    RemoveChild(NodeId, NodeId),
    AddNewParent(NodeId, NodeId, NodeData),

    ReplaceData(NodeId, NodeData),

    SetTranslation(NodeId, Vec3),
    SetRotation(NodeId, Quat),
    SetScale(NodeId, f32),
}

#[derive(Debug)]
struct IdGenerator {
    last_id: NodeId,
    returned_ids: HashSet<NodeId>,
}
impl IdGenerator {
    fn new() -> IdGenerator {
        IdGenerator {
            last_id: NodeId(0),
            returned_ids: HashSet::new(),
        }
    }

    pub fn generate(&mut self) -> NodeId {
        if !self.returned_ids.is_empty() {
            let id = *self.returned_ids.iter().next().unwrap();
            self.returned_ids.remove(&id);
            id
        } else {
            let id = self.last_id;
            self.last_id.0 += 1;
            id
        }
    }
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: HashMap<NodeId, Node>,
    pub root_node_id: NodeId,

    id_generator: IdGenerator,
}
impl Graph {
    pub fn new(data: NodeData) -> Graph {
        let mut nodes = HashMap::new();
        let mut id_generator = IdGenerator::new();
        let root_node_id = Self::add_direct(&mut nodes, &mut id_generator, data);
        Graph {
            nodes,
            root_node_id,

            id_generator,
        }
    }

    fn add_direct(
        nodes: &mut HashMap<NodeId, Node>,
        id_generator: &mut IdGenerator,
        data: NodeData,
    ) -> NodeId {
        let id = id_generator.generate();
        nodes.insert(id, Node::new(id, data));
        id
    }

    pub fn add(&mut self, data: NodeData) -> NodeId {
        Self::add_direct(&mut self.nodes, &mut self.id_generator, data)
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    fn find_all_reachable_nodes_opt(&self, node_id: Option<NodeId>, seen: &mut HashSet<NodeId>) {
        if let Some(node_id) = node_id {
            self.find_all_reachable_nodes(node_id, seen)
        }
    }

    fn find_all_reachable_nodes(&self, node_id: NodeId, seen: &mut HashSet<NodeId>) {
        let node = self.get(node_id).unwrap();
        seen.insert(node.id);
        match &node.data {
            NodeData::Sphere(_) => {}
            NodeData::Cylinder(_) => {}
            NodeData::Torus(_) => {}

            NodeData::Union(Union { children, .. }) => {
                for child_id in children {
                    self.find_all_reachable_nodes(*child_id, seen);
                }
            }
            NodeData::Intersect(Intersect {
                children: (lhs, rhs),
                ..
            }) => {
                self.find_all_reachable_nodes_opt(*lhs, seen);
                self.find_all_reachable_nodes_opt(*rhs, seen);
            }
            NodeData::Subtract(Subtract { children, .. }) => {
                for child_id in children {
                    self.find_all_reachable_nodes(*child_id, seen);
                }
            }

            NodeData::Rgb(Rgb { child, .. }) => {
                self.find_all_reachable_nodes_opt(*child, seen);
            }

            NodeData::Translate(Translate { child, .. }) => {
                self.find_all_reachable_nodes_opt(*child, seen);
            }
            NodeData::Rotate(Rotate { child, .. }) => {
                self.find_all_reachable_nodes_opt(*child, seen);
            }
            NodeData::Scale(Scale { child, .. }) => {
                self.find_all_reachable_nodes_opt(*child, seen);
            }
        }
    }

    fn garbage_collect(&mut self) {
        let all: HashSet<_> = self.nodes.keys().copied().collect();
        let mut seen = HashSet::new();
        self.find_all_reachable_nodes(self.root_node_id, &mut seen);

        for node_id in all.difference(&seen) {
            self.nodes.remove(node_id);
        }
    }

    fn apply_event(&mut self, event: &GraphEvent) -> Option<()> {
        match event {
            GraphEvent::AddChild(parent_id, index, node_data) => {
                let child_id = self.add(node_data.clone());
                self.get_mut(*parent_id)?.data.add_child(*index, child_id);
            }
            GraphEvent::RemoveChild(parent_id, child_id) => {
                self.get_mut(*parent_id)?.data.remove_child(*child_id);
            }
            GraphEvent::AddNewParent(parent_id, child_id, node_data) => {
                let new_parent_id = self.add(node_data.clone());
                self.get_mut(new_parent_id)?.data.add_child(None, *child_id);

                let parent = self.get_mut(*parent_id)?;
                parent.data.replace_child(*child_id, new_parent_id);
            }

            GraphEvent::ReplaceData(node_id, data) => self.get_mut(*node_id)?.data = data.clone(),

            GraphEvent::SetTranslation(node_id, translation) => {
                self.get_mut(*node_id)?.translation = *translation;
            }
            GraphEvent::SetRotation(node_id, rotation) => {
                self.get_mut(*node_id)?.rotation = *rotation;
            }
            GraphEvent::SetScale(node_id, scale) => {
                self.get_mut(*node_id)?.scale = *scale;
            }
        }

        Some(())
    }

    pub fn apply_events(&mut self, events: &[GraphEvent]) {
        for event in events {
            self.apply_event(event)
                .expect("failed to apply event cleanly");
        }
        self.garbage_collect();
    }
}
