#![feature(async_closure)]
mod utils;

use glium::*;
use glium::uniforms::Sampler;

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


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()?;

    rt.block_on(run())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build();

    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Crytid Squad")
        .build(&event_loop);

    let model = utils::obj::parse_object(
        "assets/models/teapot.obj",
        &display,
    ).await?;

    let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)?;

    let dimensions = display.get_max_viewport_dimensions();
    let mut camera = utils::camera::Camera::new(
        [0.0, 0.0, -5.0],
        [0.0, 0.0, 0.0],
        90.0,
        dimensions.0 as f32 / dimensions.1 as f32,
        0.1,
        100.0,
    );

    let proj = camera.get_proj();
    let view = camera.get_view();

    let mut transform = utils::transform::Transform::new(
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
    );

    let image_file = std::fs::File::open("assets/textures/teapot.png")?;
    let image_reader = std::io::BufReader::new(image_file);

    let image = image::load(image_reader, image::ImageFormat::Png)?.to_rgba8();

    let image_dimensions = image.dimensions();

    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let texture = glium::texture::Texture2d::new(
        &display,
        image,
    )?;

    let texture: &'static Texture2d = unsafe { || -> &'static _ { &*(&texture as *const _) } }();

    let sampler: Sampler<'static, _>  = texture.sampled()
        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
        .minify_filter(glium::uniforms::MinifySamplerFilter::Linear);

    let mut frame_counter = 0;
    let mut last_second = std::time::Instant::now();
    let mut last_time = std::time::Instant::now();
    let start_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                // Close the window if the exit button is pressed
                winit::event::WindowEvent::CloseRequested => close(control_flow),
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    println!("{:?}", input);

                    if let Some(key) = input.virtual_keycode {
                        match key {
                            winit::event::VirtualKeyCode::Escape => close(control_flow),
                            _ => (),
                        }
                    }
                },
                // Resize the window if the size has changed
                winit::event::WindowEvent::Resized(physical_size) => {
                    camera.aspect_ratio = physical_size.width as f32 / physical_size.height as f32;
                },
                _ => (),
            },
            winit::event::Event::RedrawRequested(_) => {
                // Update the time
                let t = start_time.elapsed().as_secs_f32();
                let dt = last_time.elapsed().as_secs_f32();

                let now = std::time::Instant::now();
                last_time = now;
                frame_counter += 1;

                if now.duration_since(last_second).as_secs_f32() >= 1.0 {
                    window.set_title(&format!("Crytid Squad - FPS: {}", frame_counter));

                    frame_counter = 0;
                    last_second = now;
                }

                let mut frame = display.draw();

                //rotate the model
                transform.rotation[1] = t * 0.5;
                transform.rotation[0] = t * 0.3;

                frame.clear_color(0.0, 0.0, 0.0, 1.0);

                let uniforms = uniform! {
                    model: transform.get_model().0,
                    view: camera.get_view(),
                    proj: camera.get_proj(),

                    texture: sampler,
                    light: [0.0, 0.0, 1.0f32],
                    ambient: 0.3f32,
                };

                frame.draw(&model.vertices, &model.indices, &program,
                    &uniforms, &Default::default()
                ).unwrap();
                frame.finish().unwrap();
            },
            winit::event::Event::RedrawEventsCleared => window.request_redraw(),
            _ => (),
        }
    });

    Ok(())
}

fn close(
    control_flow: &mut winit::event_loop::ControlFlow,
) {
    *control_flow = winit::event_loop::ControlFlow::Exit;
}
