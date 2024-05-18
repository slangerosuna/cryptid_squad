#![feature(async_closure)]
#![feature(downcast_unchecked)]
#![feature(sync_unsafe_cell)]

mod utils;
mod core;
mod networking;

pub use core::*;
pub use utils::*;
pub use networking::*;

use glium::*;
use glium::uniforms::Sampler;

use serde::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub worker_threads: usize,
    pub window_size: (u32, u32),
    pub window_title: String,

    pub exit_on_networking_error: bool,
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
                exit_on_networking_error: false,
            })?;

            std::fs::write("config.toml", &default_conf)?;

            default_conf
        }
    };
    let conf: Config = toml::from_str(&conf)?;
    let conf = Box::leak(Box::new(conf));
    let conf: &'static Config = unsafe { &*(conf as *const _) };

    let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(conf.worker_threads)
                .enable_all()
                .build()?;

    let rt = Box::leak(Box::new(rt));
    let rt: &'static tokio::runtime::Runtime = unsafe { &*(rt as *const _) };

    let mut scheduler = Scheduler::new(0.01);
    let mut game_state = core::GameState::new(&mut scheduler as *mut Scheduler, conf);

    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build();

    let renderer = core::RenderResource::new(&event_loop, &conf.window_title, conf.window_size)?;
    game_state.add_resource(renderer);

    let game_state_ref = &mut game_state;

    let renderer = game_state_ref.get_resource::<RenderResource>().unwrap();
    let renderer = unsafe { &*(renderer as *const RenderResource) }; // bypasses lifetime issues

    scheduler.add_system(get_render_system(), SystemType::Update);
    scheduler.add_system(get_rotate_cube_system(), SystemType::Update);
    scheduler.add_system(get_input_handler_system(), SystemType::Update);
    scheduler.add_system(get_init_networking_system(), SystemType::Init);

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

    teapot.add_component(&mut game_state, transform, Transform::get_component_type());
    teapot.add_component(&mut game_state, model, Model::get_component_type());
    teapot.add_component(&mut game_state, Texture { sampler }, Texture::get_component_type());

    teapot.add_component(&mut game_state, RenderObject, RenderObject::get_component_type());

    let camera_entity = game_state.create_entity("Camera".to_string());

    camera_entity.add_component(&mut game_state, camera, 3);

    let camera = camera_entity.get_component_mut::<Camera>(Camera::get_component_type()).unwrap();
    let camera = camera as *mut Camera;

    let input_handler = InputHandler::new();
    game_state.add_resource(input_handler);

    let input_handler = game_state.get_resource_mut::<InputHandler>().unwrap();
    let input_handler = input_handler as *mut InputHandler;

    rt.block_on(scheduler.init(&mut game_state));

    let fixed_update_scheduler = unsafe {&*(&scheduler as *const Scheduler)};
    let fixed_update_future = fixed_update_scheduler.loop_fixed_update(&mut game_state as *mut _);
    let mut fixed_update_future = unsafe { SendBox::new(fixed_update_future) };
    let fixed_update_future = unsafe { std::pin::Pin::new_unchecked(&mut *(&mut fixed_update_future as *mut _)) };
    rt.spawn(fixed_update_future);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                // Close the window if the exit button is pressed
                winit::event::WindowEvent::CloseRequested => close(control_flow, rt),
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        unsafe {&mut *input_handler}.handle_key_press(key, input.state);
                        match key {
                            winit::event::VirtualKeyCode::Escape => close(control_flow, rt),
                            _ => (),
                        }
                    }
                },
                winit::event::WindowEvent::Resized(physical_size) => {
                    unsafe {&mut *camera}.aspect_ratio = physical_size.width as f32 / physical_size.height as f32;
                },
                _ => (),
            },
            winit::event::Event::RedrawRequested(_) => {
                rt.block_on(scheduler.update(&mut game_state));
                if game_state.should_close {
                    rt.block_on(scheduler.close(&mut game_state));
                    close(control_flow, rt);
                }
            },
            winit::event::Event::RedrawEventsCleared => renderer.window.request_redraw(),
            _ => (),
        }
    });
}

struct SendBox<T>(std::pin::Pin<Box<T>>);

unsafe impl<T> Send for SendBox<T> {}

impl<T> SendBox<T> {
    unsafe fn new(t: T) -> Self {
        SendBox(Box::pin(t))
    }
}

impl<T> futures::Future for SendBox<T>
where
    T: futures::Future + 'static,
{
    type Output = T::Output;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
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
