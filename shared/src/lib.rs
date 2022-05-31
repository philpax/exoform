use anyhow::Context;
use bevy_math::prelude::*;

#[derive(Debug, PartialEq)]
pub enum Node {
    Sphere { position: Vec3, radius: f32 },
    Union(f32, Vec<Node>),
    Rgb(f32, f32, f32, Box<Node>),
}

fn get_f32_attr(node: &kdl::KdlNode, key: &str) -> anyhow::Result<f32> {
    value_to_f32(node.get(key).context(format!("expected {}", key))?.value())
}

fn value_to_f32(value: &kdl::KdlValue) -> anyhow::Result<f32> {
    value
        .as_i64()
        .map(|v| v as f32)
        .or(value.as_f64().map(|v| v as f32))
        .context("expected float")
}

fn get_children_for_operation<'a>(
    node: &'a kdl::KdlNode,
    operation: &'static str,
) -> anyhow::Result<&'a [kdl::KdlNode]> {
    Ok(node
        .children()
        .context(format!("expected children for {operation}"))?
        .nodes())
}

fn parse_nodes(nodes: &[kdl::KdlNode]) -> anyhow::Result<Vec<Node>> {
    nodes.iter().map(parse_node).collect::<Result<Vec<_>, _>>()
}

fn parse_children_for_operation(
    node: &kdl::KdlNode,
    operation: &'static str,
) -> anyhow::Result<Vec<Node>> {
    get_children_for_operation(node, operation).and_then(parse_nodes)
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
            let size = if node.get("size").is_some() {
                get_f32_attr(node, "size")?
            } else {
                0.0
            };

            let nodes = parse_children_for_operation(node, "union")?;
            Ok(Node::Union(size, nodes))
        }
        "rgb" => {
            if let [r, g, b] = node.entries() {
                let (r, g, b) = (
                    value_to_f32(r.value())?,
                    value_to_f32(g.value())?,
                    value_to_f32(b.value())?,
                );

                if let [child] = get_children_for_operation(node, "rgb")? {
                    Ok(Node::Rgb(r, g, b, Box::new(parse_node(child)?)))
                } else {
                    anyhow::bail!("expected one child for rgb")
                }
            } else {
                anyhow::bail!("expected three floats for arguments")
            }
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
                vec![
                    Node::Sphere {
                        position: Vec3::new(0.0, 0.0, 0.0),
                        radius: 1.0
                    },
                    Node::Sphere {
                        position: Vec3::new(1.0, 0.0, 0.0),
                        radius: 2.5
                    }
                ]
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

    #[test]
    fn can_parse_rgb() {
        let input = r#"
rgb 0.2 0.4 0.8 {
    sphere x=0 y=0 z=0 r=1
}
"#;

        assert_eq!(
            code_to_node(input).ok(),
            Some(Node::Rgb(
                0.2,
                0.4,
                0.8,
                Box::new(Node::Sphere {
                    position: Vec3::new(0.0, 0.0, 0.0),
                    radius: 1.0
                })
            ))
        );
    }
}
