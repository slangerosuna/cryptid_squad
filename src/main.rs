use tokio::main;
use glium::*;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, color);

const vertex_shader_src: &str = r#"
    #version 140

    in vec2 position;
    in vec3 color;
    out vec3 vColor;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        vColor = color;
    }
"#;

const fragment_shader_src: &str = r#"
    #version 140

    in vec3 vColor;
    out vec4 color;

    void main() {
        color = vec4(vColor, 1.0);
    }
"#;


#[main(
    flavor = "multi_thread",
    worker_threads = 8,
)]
async fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build();

    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Crytid Squad")
        .build(&event_loop);

    let vertex1 = Vertex {
        position: [0.0, 0.0],
        color: [1.0, 0.0, 0.0],
    };
    let vertex2 = Vertex {
        position: [0.0,  0.5],
        color: [0.0, 1.0, 0.0],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
        color: [0.0, 0.0, 1.0],
    };

    let shape = vec![vertex1, vertex2, vertex3];
    
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    loop {
        let mut frame = display.draw();
        frame.clear_color(0.0, 0.0, 1.0, 1.0);
        frame.draw(&vertex_buffer, &indices, &program, 
            &glium::uniforms::EmptyUniforms, &Default::default()
        ).expect("Failed to draw");
        frame.finish()
            .expect("Failed to swap buffers");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = winit::event_loop::ControlFlow::Poll;
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    // Close the window if the exit button is pressed
                    winit::event::WindowEvent::CloseRequested => *control_flow = winit::event_loop::ControlFlow::Exit,
                    _ => (),
                },
                _ => (),
            }
        });
    }
}
