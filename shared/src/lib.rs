use anyhow::Context;
use bevy_math::prelude::*;

#[derive(Debug, PartialEq)]
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
    Intersect(f32, (Box<Node>, Box<Node>)),
    Subtract(f32, (Box<Node>, Box<Node>)),
    Rgb(f32, f32, f32, Box<Node>),
}

fn value_to_f32(value: &kdl::KdlValue) -> anyhow::Result<f32> {
    value
        .as_i64()
        .map(|v| v as f32)
        .or(value.as_f64().map(|v| v as f32))
        .context("expected float")
}

fn entry_to_f32(entry: &kdl::KdlEntry) -> anyhow::Result<f32> {
    value_to_f32(entry.value())
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

fn parse_lhs_rhs_for_operation(
    node: &kdl::KdlNode,
    operation: &'static str,
) -> anyhow::Result<(Box<Node>, Box<Node>)> {
    let children = get_children_for_operation(node, operation)?;
    if children.len() != 2 {
        anyhow::bail!("expected two children for {operation}");
    }
    Ok((
        Box::new(parse_node(&children[0])?),
        Box::new(parse_node(&children[1])?),
    ))
}

fn parse_node(node: &kdl::KdlNode) -> anyhow::Result<Node> {
    match node.name().value() {
        "sphere" => {
            let entries = node.entries();
            if entries.len() != 4 {
                anyhow::bail!("expected x y z r for sphere");
            }

            let x = entry_to_f32(&entries[0])?;
            let y = entry_to_f32(&entries[1])?;
            let z = entry_to_f32(&entries[2])?;
            let r = entry_to_f32(&entries[3])?;

            Ok(Node::Sphere {
                position: Vec3::new(x, y, z),
                radius: r,
            })
        }
        "cylinder" => {
            let entries = node.entries();
            if entries.len() != 3 {
                anyhow::bail!("expected cylinder_radius half_height rounding_radius for cylinder");
            }

            let cylinder_radius = entry_to_f32(&entries[0])?;
            let half_height = entry_to_f32(&entries[1])?;
            let rounding_radius = entry_to_f32(&entries[2])?;

            Ok(Node::RoundedCylinder {
                cylinder_radius,
                half_height,
                rounding_radius,
            })
        }
        "torus" => {
            let entries = node.entries();
            if entries.len() != 2 {
                anyhow::bail!("expected big_r small_r for torus");
            }

            let big_r = entry_to_f32(&entries[0])?;
            let small_r = entry_to_f32(&entries[1])?;

            if big_r < small_r {
                anyhow::bail!("expected big_r to be larger than small_r for torus");
            }

            Ok(Node::Torus { big_r, small_r })
        }
        "union" => {
            let size = node
                .entries()
                .get(0)
                .map(entry_to_f32)
                .transpose()?
                .unwrap_or_default();

            let nodes = parse_children_for_operation(node, "union")?;
            Ok(Node::Union(size, nodes))
        }
        "intersect" => {
            let size = node
                .entries()
                .get(0)
                .map(entry_to_f32)
                .transpose()?
                .unwrap_or_default();

            let nodes = parse_lhs_rhs_for_operation(node, "intersect")?;
            Ok(Node::Intersect(size, nodes))
        }
        "subtract" => {
            let size = node
                .entries()
                .get(0)
                .map(entry_to_f32)
                .transpose()?
                .unwrap_or_default();

            let nodes = parse_lhs_rhs_for_operation(node, "subtract")?;
            Ok(Node::Subtract(size, nodes))
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
union 0.4 {
    sphere 0 0 0 1
    sphere 1 0 0 2.5
    sphere 0.5 0 0.5 2.5
}
"#;

        assert_eq!(
            code_to_node(input).ok(),
            Some(Node::Union(
                0.4,
                vec![
                    Node::Sphere {
                        position: Vec3::new(0.0, 0.0, 0.0),
                        radius: 1.0
                    },
                    Node::Sphere {
                        position: Vec3::new(1.0, 0.0, 0.0),
                        radius: 2.5
                    },
                    Node::Sphere {
                        position: Vec3::new(0.5, 0.0, 0.5),
                        radius: 2.5
                    }
                ]
            ))
        );
    }

    #[test]
    fn fails_on_multiple_root_nodes() {
        let input = r#"
sphere 0 0 0 1
sphere 1 0 0 2.5
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
    sphere 0 0 0 1
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

    #[test]
    fn can_parse_intersection() {
        let input = r#"
intersect {
    sphere 0 0 0 1
    sphere 1 0 0 2.5
}
"#;

        assert_eq!(
            code_to_node(input).ok(),
            Some(Node::Intersect(
                0.0,
                (
                    Box::new(Node::Sphere {
                        position: Vec3::new(0.0, 0.0, 0.0),
                        radius: 1.0
                    }),
                    Box::new(Node::Sphere {
                        position: Vec3::new(1.0, 0.0, 0.0),
                        radius: 2.5
                    }),
                )
            ))
        );
    }

    #[test]
    fn can_parse_rounded_cylinder() {
        let input = "cylinder 0.5 1.0 0.1";
        assert_eq!(
            code_to_node(input).ok(),
            Some(Node::RoundedCylinder {
                cylinder_radius: 0.5,
                half_height: 1.0,
                rounding_radius: 0.1,
            })
        )
    }

    #[test]
    fn can_parse_torus() {
        let input = "torus 2.0 1.0";
        assert_eq!(
            code_to_node(input).ok(),
            Some(Node::Torus {
                big_r: 2.0,
                small_r: 1.0,
            })
        )
    }
}
