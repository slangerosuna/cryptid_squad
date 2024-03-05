use crate::core::*;
use crate::utils::obj::Model;
use crate::utils::transform::Transform;
use crate::utils::camera::Camera;
use glium::*;
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
        float intensity = clamp(dot(vNormal, light), 0.0, 1.0 - ambient) + ambient;

        color = texture2D(texture, vUv) * vec4(intensity, intensity, intensity, 1.0);
    }
"#;

// component type 0
#[derive(Debug)]
pub struct RenderResource {
    pub window: winit::window::Window,
    pub display: glium::Display<glutin::surface::WindowSurface>,
    pub program: glium::Program,
}
impl_resource!(RenderResource, 0);

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

impl RenderResource {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        window_title: &str,
        inner_size: (u32, u32),
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(inner_size.0, inner_size.1)
            .build(&event_loop);
        let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)?;

        Ok(Self{
            window,
            display,
            program,
        })
    }
}

create_system!(render, get_render_system;
    uses RenderResource, RenderObject, Texture, Model, Transform, Camera);
async fn render(game_state: &mut GameState, t: f64, _dt: f64) {
    let render_resource = game_state.get_resource::<RenderResource>().unwrap();

    let mut frame = render_resource.display.draw();
    let camera = game_state.components[3][0].get();
    let camera = &unsafe { &*camera }.component;
    let camera = unsafe { camera.as_any().downcast_ref_unchecked::<crate::utils::camera::Camera>() };

    for render_object in game_state.components[RenderObject::get_component_type()].iter() {
        let render_object = render_object.get();
        let render_object = unsafe { &*render_object };

        let id = render_object.owner as usize;
        let entity = game_state.get_entity_mut(id).unwrap();

        {
            let transform = entity.get_component_mut::<Transform>(Transform::get_component_type()).await.unwrap();

            //rotate the model
            transform.rotation[1] = (t * 0.5) as f32;
            transform.rotation[0] = (t * 0.3) as f32;
        }
        let transform = entity.get_component::<Transform>(Transform::get_component_type()).await.unwrap();
        let model = entity.get_component::<Model>(Model::get_component_type()).await.unwrap();
        let texture = entity.get_component::<Texture>(Texture::get_component_type()).await.unwrap();

        let sampler = texture.sampler;


        frame.clear_color(0.0, 0.0, 0.0, 1.0);

        let uniforms = uniform! {
            model: transform.get_model().0,
            view: camera.get_view(),
            proj: camera.get_proj(),

            texture: sampler,
            ambient: 0.3f32,
            light: crate::utils::math::normalize([-1.0, 0.4, 0.9f32]),
        };

        if let Err(e) = frame.draw(&model.vertices, &model.indices, &render_resource.program, &uniforms, &Default::default())
            { eprintln!("Error drawing frame: {}", e.to_string()); return; }
    }

    if let Err(e) = frame.finish() { eprintln!("Error finishing frame: {}", e.to_string()); }
}
