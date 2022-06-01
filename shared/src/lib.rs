use bevy_math::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Sphere {
        position: Vec3,
        radius: f32,
    },
    RoundedCylinder {
        cylinder_radius: f32,
        half_height: f32,
        rounding_radius: f32,
    },
    Torus {
        big_r: f32,
        small_r: f32,
    },
    Union(f32, Vec<Node>),
    Intersect(f32, (Option<Box<Node>>, Option<Box<Node>>)),
    Subtract(f32, (Option<Box<Node>>, Option<Box<Node>>)),
    Rgb(f32, f32, f32, Option<Box<Node>>),
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
