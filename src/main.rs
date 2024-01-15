mod utils;

use glium::*;

const VERTEX_SHADER_SRC: &str = r#"
    #version 140
    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 proj;

    in vec3 position;
    in vec3 normal;
    in vec2 uv;

    out vec3 vNormal;
    out vec2 vUv;

    void main() {
        vNormal = normal;
        vUv = uv;

        gl_Position = proj * view * model * vec4(position, 1.0);
    }
"#;

const FRAGMENT_SHADER_SRC: &str = r#"
    #version 140
    uniform sampler2D texture;
    uniform vec3 light;
    uniform float ambient;

    in vec3 vNormal;
    in vec2 vUv;

    out vec4 color;

    void main() {
        float intensity = clamp(dot(vNormal, light), 0.0, 1.0 - ambient) + ambient;

        color = texture2D(texture, vUv) * vec4(intensity, intensity, intensity, 1.0);
    }
"#;


#[tokio::main(
    flavor = "multi_thread",
    worker_threads = 8,
)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let mut camera = utils::camera::Camera::new(
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        90.0,
        1.0,
        0.1,
        100.0,
    );

    let proj = camera.get_proj();
    let view = camera.get_view();

    let transform: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0 ,0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, -1.0, -1.0, 1.0],
    ];

    let texture = glium::texture::Texture2d::new(
        &display,
        utils::image::load(
            "../assets/textures/teapot.png".to_string(),
        ).unwrap()
    ).unwrap();

    let sampler = texture.sampled()
        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
        .minify_filter(glium::uniforms::MinifySamplerFilter::Linear);

    loop {
        let mut frame = display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);

        let uniforms = uniform! {
            model: transform,
            view: view,
            proj: proj,
            texture: sampler,
            light: [0.0, 0.0, 1.0f32],
            ambient: 0.3f32,
        };

        frame.draw(&model.vertices, &model.indices, &program,
            &uniforms, &Default::default()
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

    Ok(())
}

fn close(
    control_flow: &mut winit::event_loop::ControlFlow,
) {
    *control_flow = winit::event_loop::ControlFlow::Exit;

    std::process::exit(0);
}
