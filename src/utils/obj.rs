use crate::core::*;
use glium::{glutin::surface::WindowSurface, *};
use std::{any::Any, collections::HashMap};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

implement_vertex!(Vertex, position, normal, uv);

// component type 5
#[derive(Debug)]
pub struct Model {
    pub vertices: VertexBuffer<Vertex>,
    pub indices: IndexBuffer<u32>,
}
impl_component!(Model, 5);

macro_rules! err {
    //creates a macro that returns an error
    () => {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid obj file")
    };
}

pub async fn parse_object(
    path: &str,
    display: &glium::Display<WindowSurface>,
) -> Result<Model, Box<dyn std::error::Error>> {
    let mut vertices: Vec<(u32, u32, u32)> = Vec::new(); // (position, normal, uv)
    let mut vertex_index_map: HashMap<(u32, u32, u32), u32> = HashMap::new();
    let mut indices: Vec<u32> = Vec::new();

    let contents = std::fs::read_to_string(path)?;

    let lines = contents.lines();

    let mut vertex_positions: Vec<[f32; 3]> = Vec::new();
    let mut vertex_normals: Vec<[f32; 3]> = Vec::new();
    let mut vertex_uvs: Vec<[f32; 2]> = Vec::new();

    for line in lines {
        // skips empty lines and comments
        if line.is_empty() || line.as_bytes()[0] == "#".as_bytes()[0] {
            continue;
        }

        // removes comments from the line
        let line = line.split("#").next().unwrap();

        let mut words = line.split_whitespace();

        match words.next() {
            Some("v") => {
                vertex_positions.push([
                    words.next().ok_or(err!())?.parse()?,
                    words.next().ok_or(err!())?.parse()?,
                    words.next().ok_or(err!())?.parse()?,
                ]);
            }
            Some("f") => {
                let f: Vec<[u32; 3]> = words
                    .map(|x| //maps each word to a [u32; 3]
                    x
                        .split("/")
                        .map(|y| y.parse::<u32>().unwrap() - 1) //subtracts 1 from each index to make it 0-based instead of 1-based
                        .collect::<Vec<u32>>()
                        .try_into()
                        .unwrap())
                    .collect();

                let handle_vertex = |x: [u32; 3]| {
                    let x = (x[0], x[1], x[2]);

                    if let Some(index) = vertex_index_map.get(&x) {
                        indices.push(*index);
                        return;
                    }

                    let index = vertices.len() as u32;
                    vertices.push(x);
                    indices.push(index);
                    vertex_index_map.insert(x, index);
                };
                match f.len() {
                    3 => f.into_iter().for_each(handle_vertex),
                    4 => {
                        //converts the quad into two triangles
                        [f[0], f[1], f[2], f[0], f[2], f[3]]
                            .into_iter()
                            .for_each(handle_vertex);
                    }
                    _ => return Err(Box::new(err!())),
                }
            }
            Some("vt") => {
                vertex_uvs.push([
                    words.next().ok_or(err!())?.parse()?,
                    words.next().ok_or(err!())?.parse()?,
                ]);
            }
            Some("vn") => {
                vertex_normals.push([
                    words.next().ok_or(err!())?.parse()?,
                    words.next().ok_or(err!())?.parse()?,
                    words.next().ok_or(err!())?.parse()?,
                ]);
            }
            _ => (),
        }
    }

    let vertices = vertices
        .iter()
        .map(|x| {
            let position = vertex_positions[x.0 as usize];
            let normal = vertex_normals[x.2 as usize];
            let uv = vertex_uvs[x.1 as usize];

            Vertex {
                position,
                normal,
                uv,
            }
        })
        .collect::<Vec<Vertex>>();

    Ok(Model {
        vertices: VertexBuffer::new(display, &vertices)?,
        indices: IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &indices,
        )?,
    })
}
