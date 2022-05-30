use anyhow::Context;
use bevy_math::prelude::*;

#[derive(Debug, PartialEq)]
pub enum Node {
    Sphere { position: Vec3, radius: f32 },
    Union(f32, Box<Node>, Box<Node>),
}

fn get_f32_attr(node: &kdl::KdlNode, key: &str) -> anyhow::Result<f32> {
    let value = node.get(key).context(format!("expected {}", key))?.value();

    value
        .as_i64()
        .map(|v| v as f32)
        .or(value.as_f64().map(|v| v as f32))
        .context("expected float")
}

fn parse_node(node: &kdl::KdlNode) -> anyhow::Result<Node> {
    match node.name().value() {
        "sphere" => {
            let x = get_f32_attr(node, "x")?;
            let y = get_f32_attr(node, "y")?;
            let z = get_f32_attr(node, "z")?;
            let r = get_f32_attr(node, "r")?;

            Ok(Node::Sphere {
                position: Vec3::new(x, y, z),
                radius: r,
            })
        }
        "union" => {
            let children = node
                .children()
                .context("expected children for union")?
                .nodes();
            if children.len() != 2 {
                anyhow::bail!("expected two children for union");
            }

            let size = if node.get("size").is_some() {
                get_f32_attr(node, "size")?
            } else {
                0.0
            };

            Ok(Node::Union(
                size,
                Box::new(parse_node(&children[0])?),
                Box::new(parse_node(&children[1])?),
            ))
        }
        _ => anyhow::bail!("unsupported node type"),
    }
}

pub fn code_to_node(description: &str) -> anyhow::Result<Node> {
    let doc: kdl::KdlDocument = description.parse()?;

    if doc.nodes().len() != 1 {
        anyhow::bail!("expected only one root node")
    }

    parse_node(&doc.nodes()[0])
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn can_parse_basic_union() {
        let input = r#"
union {
    sphere x=0 y=0 z=0 r=1
    sphere x=1 y=0 z=0 r=2.5
}
"#;

        assert_eq!(
            code_to_node(input).ok(),
            Some(Node::Union(
                0.0,
                Box::new(Node::Sphere {
                    position: Vec3::new(0.0, 0.0, 0.0),
                    radius: 1.0
                }),
                Box::new(Node::Sphere {
                    position: Vec3::new(1.0, 0.0, 0.0),
                    radius: 2.5
                })
            ))
        );
    }

    #[test]
    fn fails_on_multiple_root_nodes() {
        let input = r#"
sphere x=0 y=0 z=0 r=1
sphere x=1 y=0 z=0 r=2.5
"#;

        assert_eq!(
            code_to_node(input).err().unwrap().to_string(),
            "expected only one root node"
        );
    }
}
