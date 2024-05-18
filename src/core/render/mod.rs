use crate::*;
use std::ffi::CString;
use glium::{backend::Facade, *};
use glutin::display::{Display, GlDisplay};
use glutin::surface::WindowSurface;
use std::any::Any;

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
        vNormal = (model * vec4(normal, 0.0)).xyz;
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
        float intensity = clamp(dot(vNormal, light), 0.0, 1.0) * (1.0 - ambient) + ambient;

        color = texture2D(texture, vUv) * vec4(intensity, intensity, intensity, 1.0);
    }
"#;

// component type 0
#[derive(Debug)]
pub struct RenderResource<'a> {
    pub window: winit::window::Window,
    pub display: glium::Display<WindowSurface>,
    pub program: glium::Program,
    pub params: glium::DrawParameters<'a>,
}
impl_resource!(RenderResource<'static>, 0);

// component type 1
#[derive(Debug)]
pub struct RenderObject;
impl_component!(RenderObject, 1);

// component type 4
#[derive(Debug)]
pub struct Texture<'a> {
    pub sampler: glium::uniforms::Sampler<'a, glium::texture::Texture2d>,
}
impl_component!(Texture<'static>, 4);

impl RenderResource<'_> {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        window_title: &str,
        inner_size: (u32, u32),
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(inner_size.0, inner_size.1)
            .build(event_loop);

        let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)?;

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };
        Ok(Self{
            window,
            display,
            program,
            params,
        })
    }
}

const SPEED: f32 = 5.0;

create_system!(rotate_cube, get_rotate_cube_system;
    uses Transform, InputHandler);
async fn rotate_cube(game_state: &mut GameState, t: f64, dt: f64) {
    let input = game_state.get_resource::<InputHandler>().unwrap();
    let forward =
        if input.is_down(winit::event::VirtualKeyCode::W) { dt as f32 * SPEED }
        else if input.is_down(winit::event::VirtualKeyCode::S) { -dt as f32 * SPEED }
        else { 0.0 };

    let right =
        if input.is_down(winit::event::VirtualKeyCode::A) { -dt as f32 * SPEED }
        else if input.is_down(winit::event::VirtualKeyCode::D) { dt as f32 * SPEED }
        else { 0.0 };

    for transform in game_state
                        .get_components_mut::<Transform>
                            (Transform::get_component_type())
                        .iter_mut()
    {
        transform.rotation[1] = (t * 0.5) as f32;
        transform.rotation[0] = (t * 0.3) as f32;

        transform.position[0] += right;
        transform.position[1] += forward;
    }
}

//uses GameState to ensure that it can unlock the scheduler lock
create_system!(render, get_render_system;
    uses GameState, RenderResource, RenderObject, Texture, Model, Transform, Camera);
async fn render(game_state: &mut GameState, _t: f64, _dt: f64) {
    let render_resource = game_state.get_resource::<RenderResource>().unwrap();

    // unlocks global scheduler lock to allow for FixedUpdate to run while waiting for vsync
    unsafe { (&*game_state.scheduler).force_unlock().await; }

    let mut frame = render_resource.display.draw();
    frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

    unsafe { (&*game_state.scheduler).force_lock().await; }

    let camera: &Camera
        = game_state.get_components
            (Camera::get_component_type())[0];

    for entity in
        game_state
            .get_entities_with::<RenderObject>
                (RenderObject::get_component_type())
            .iter()
    {
        let transform = entity.get_component::<Transform>(Transform::get_component_type());
        let transform = if let Some(transform) = transform { transform } else { continue; };

        let model = entity.get_component::<Model>(Model::get_component_type());
        let model = if let Some(model) = model { model } else { continue; };

        let texture = entity.get_component::<Texture>(Texture::get_component_type());
        let texture = if let Some(texture) = texture { texture } else { continue; };

        let sampler = texture.sampler;

        let uniforms = uniform! {
            model: transform.get_model().0,
            view: camera.get_view(),
            proj: camera.get_proj(),

            texture: sampler,
            ambient: 0.3f32,
            light: normalize([-1.0, 5.0, 0.9]),
        };


        if let Err(e) = frame.draw(&model.vertices, &model.indices, &render_resource.program, &uniforms, &render_resource.params)
            { eprintln!("Error drawing frame: {}", e.to_string()); return; }
    }

    if let Err(e) = frame.finish() { eprintln!("Error finishing frame: {}", e.to_string()); }
}
