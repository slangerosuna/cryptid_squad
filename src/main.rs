mod utils;

use glium::*;

const VERTEX_SHADER_SRC: &str = r#"
    #version 140

    in vec2 position;
    in vec3 color;
    out vec3 vColor;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        vColor = color;
    }
"#;

const FRAGMENT_SHADER_SRC: &str = r#"
    #version 140
    uniform Texture tex;

    in vec3 vColor;
    out vec4 color;

    void main() {
        color = vec4(vColor, 1.0);
    }
"#;


#[tokio::main(
    flavor = "multi_thread",
    worker_threads = 8,
)]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build();

    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Crytid Squad")
        .build(&event_loop);

    let model = utils::obj::parse_object(
        "assets/models/teapot.obj",
        &display,
    ).await?;

    let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    loop {
        let mut frame = display.draw();
        frame.clear_color(0.0, 0.0, 1.0, 1.0);
        frame.draw(&model.vertices, &model.indices, &program, 
            &glium::uniforms::EmptyUniforms, &Default::default()
        ).expect("Failed to draw");
        frame.finish()
            .expect("Failed to swap buffers");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = winit::event_loop::ControlFlow::Poll;
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    // Close the window if the exit button is pressed
                    winit::event::WindowEvent::CloseRequested => close(control_flow),
                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(key) = input.virtual_keycode {
                            match key {
                                winit::event::VirtualKeyCode::Escape => close(control_flow),
                                _ => (),
                            }
                        }
                    },
                    _ => (),
                },
                _ => (),
            }
        });
    }
}

fn close(
    control_flow: &mut winit::event_loop::ControlFlow,
) {
    *control_flow = winit::event_loop::ControlFlow::Exit;
}