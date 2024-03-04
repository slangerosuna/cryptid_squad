#![feature(async_closure)]
#[allow(dead_code)]
mod utils;
mod core;

use core::*;

use glium::*;
use glium::uniforms::Sampler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()?;

    rt.block_on(run())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut game_state = core::GameState::new(0.01);

    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build();
    let renderer = core::RenderResource::new(&event_loop)?;
    game_state.add_resource(renderer);

    let renderer = game_state.get_resource::<RenderResource>().unwrap();
    game_state.scheduler.add_system(render_system, SystemType::Update);

    let model = utils::obj::parse_object(
        "assets/models/teapot.obj",
        &renderer.display,
    ).await?;


    let dimensions = &renderer.display.get_max_viewport_dimensions();
    let mut camera = utils::camera::Camera::new(
        [0.0, 0.0, -5.0],
        [0.0, 0.0, 0.0],
        90.0,
        dimensions.0 as f32 / dimensions.1 as f32,
        0.1,
        100.0,
    );

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
        &renderer.display,
        image,
    )?;

    // gives texture a static lifetime to allow it to be used in the draw loop
    let texture: &'static Texture2d = unsafe { &*(&texture as *const _) };

    let sampler: Sampler<'static, _>  = texture.sampled()
        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
        .minify_filter(glium::uniforms::MinifySamplerFilter::Linear);

    let mut input_handler = utils::input_handling::InputHandler::new();

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
                    if let Some(key) = input.virtual_keycode {
                        input_handler.handle_key_press(key, input.state);
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
                    //window.set_title(&format!("Crytid Squad - FPS: {}", frame_counter));

                    frame_counter = 0;
                    last_second = now;
                }

                input_handler.periodic();

            },
            winit::event::Event::RedrawEventsCleared => renderer.window.request_redraw(),
            _ => (),
        }
    });
}

fn close(
    control_flow: &mut winit::event_loop::ControlFlow,
) {
    *control_flow = winit::event_loop::ControlFlow::Exit;
}
