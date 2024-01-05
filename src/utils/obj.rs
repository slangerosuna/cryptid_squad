use glium::{
    glutin::surface::WindowSurface,
    *,
};

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

implement_vertex!(Vertex, position, uv);

pub struct Model {
    pub vertices: VertexBuffer<Vertex>,
    pub indices: IndexBuffer<u32>,
}

pub async fn parse_object( 
    path: &str,
    display: &glium::Display<WindowSurface>,
) -> Result<Model, Box<dyn std::error::Error>> {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let contents = std::fs::read_to_string(path)
        .expect("Could not read obj file");

    let lines = contents.lines();
    
    let mut ui: usize = 0;
    let mut ni: usize = 0;

    let err = || { std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid obj file",
    )};

    for line in lines {
        let mut words = line.split_whitespace();

        match words.next() {
            Some("v") => {
                // TODO: stop using unwrap, make it return an Err instead
                let x: f32 = words.next().ok_or(err())?.parse()?;
                let y: f32 = words.next().ok_or(err())?.parse()?;
                let z: f32 = words.next().ok_or(err())?.parse()?;

                vertices.push(Vertex {
                    position: [x, y, z],
                    uv: [0.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                });
            },
            Some("f") => {
                let v1: u32 = words.next().ok_or(err())?.parse()?;
                let v2: u32 = words.next().ok_or(err())?.parse()?;
                let v3: u32 = words.next().ok_or(err())?.parse()?;

                indices.push(v1 - 1);
                indices.push(v2 - 1);
                indices.push(v3 - 1);
            },
            Some("vt") => {
                let u: f32 = words.next().ok_or(err())?.parse()?;
                let v: f32 = words.next().ok_or(err())?.parse()?;
                
                vertices[ui].uv = [u, v];
                ui += 1;
            },
            Some("vn") => {
                let x: f32 = words.next().ok_or(err())?.parse()?;
                let y: f32 = words.next().ok_or(err())?.parse()?;
                let z: f32 = words.next().ok_or(err())?.parse()?;

                vertices[ni].normal = [x, y, z];
                ni += 1;
            },
            _ => (),
        }
    }

    Ok(Model {
        vertices: VertexBuffer::new(display, &vertices)?,
        indices: IndexBuffer::new(
            display, 
            glium::index::PrimitiveType::TrianglesList,
            &indices
        )?,
    })
}