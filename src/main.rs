#![feature(async_closure)]
#![feature(downcast_unchecked)]
mod utils;
mod core;

use core::*;

use glium::*;
use glium::uniforms::Sampler;

use serde::*;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    pub worker_threads: usize,
    pub window_size: (u32, u32),
    pub window_title: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = std::fs::read_to_string("config.toml");
    let conf = match conf {
        Ok(conf) => conf,
        Err(e) => {
            println!("Error reading config file: {}", e);

            let default_conf = toml::to_string(&Config {
                worker_threads: 4,
                window_size: (800, 600),
                window_title: "Cryptid Squad".to_string(),
            })?;

            std::fs::write("config.toml", default_conf)?;
            std::process::exit(1);
        }
    };
    let conf: Config = toml::from_str(&conf)?;

    let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(conf.worker_threads)
                .enable_all()
                .build()?;

    let rt = Box::leak(Box::new(rt));
    let rt: &'static tokio::runtime::Runtime = unsafe { &*(rt as *const _) };

    let mut game_state = core::GameState::new();
    let mut scheduler = Scheduler::new(0.01);

    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build();

    let renderer = core::RenderResource::new(&event_loop, &conf.window_title, conf.window_size)?;
    game_state.add_resource(renderer);

    let game_state_ref = &mut game_state;

    let renderer = game_state_ref.get_resource::<RenderResource>().unwrap();
    let renderer = unsafe { &*(renderer as *const RenderResource) }; // bypasses lifetime issues

    scheduler.add_system(get_render_system(), SystemType::Update);

    let model = rt.block_on(utils::obj::parse_object(
        "assets/models/teapot.obj",
        &renderer.display,
    ))?;


    let dimensions = &renderer.display.get_max_viewport_dimensions();
    let camera = utils::camera::Camera::new(
        [0.0, 0.0, -5.0],
        [0.0, 0.0, 0.0],
        90.0,
        dimensions.0 as f32 / dimensions.1 as f32,
        0.1,
        100.0,
    );

    let transform = utils::transform::Transform::new(
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

    let teapot = game_state.create_entity("Teapot".to_string());

    teapot.add_component(&mut game_state, transform, utils::transform::Transform::get_component_type());
    teapot.add_component(&mut game_state, model, utils::obj::Model::get_component_type());
    teapot.add_component(&mut game_state, Texture { sampler }, Texture::get_component_type());

    teapot.add_component(&mut game_state, core::RenderObject, core::RenderObject::get_component_type());

    let camera_entity = game_state.create_entity("Camera".to_string());

    camera_entity.add_component(&mut game_state, camera, 3);

    // safe to get the component mut here because we don't mutate during the init, update, or fixed_update phases
    let camera = rt.block_on(camera_entity.get_component_mut::<utils::camera::Camera>(utils::camera::Camera::get_component_type())).unwrap();

    let mut input_handler = utils::input_handling::InputHandler::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                // Close the window if the exit button is pressed
                winit::event::WindowEvent::CloseRequested => close(control_flow, rt),
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        input_handler.handle_key_press(key, input.state);
                        match key {
                            winit::event::VirtualKeyCode::Escape => close(control_flow, rt),
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
                input_handler.periodic();
                rt.block_on(scheduler.update(&mut game_state));
            },
            winit::event::Event::RedrawEventsCleared => renderer.window.request_redraw(),
            _ => (),
        }
    });
}

fn close(
    control_flow: &mut winit::event_loop::ControlFlow,
    rt: &'static tokio::runtime::Runtime,
) {
    *control_flow = winit::event_loop::ControlFlow::Exit;

    // drop the runtime to ensure all tasks are finished
    unsafe { std::ptr::drop_in_place(rt as *const _ as *mut tokio::runtime::Runtime); }

    std::process::exit(0);
}