use crate::core::*;
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
pub struct RenderResource {
    pub window: winit::window::Window,
    pub display: glium::Display<glutin::surface::WindowSurface>,
    pub program: glium::Program,
}
impl_resource!(RenderResource);

impl RenderResource {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
            .with_title("Crytid Squad")
            .build(&event_loop);
        let program = glium::Program::from_source(&display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)?;

        Ok(Self{
            window,
            display,
            program,
        })
    }
}

pub static render_system: System = System {
    system: render,
    args: vec![0],
};

fn render(game_state: &mut GameState) {
    let render_resource = game_state.get_resource::<RenderResource>().unwrap();

    let mut frame = render_resource.display.draw();

    //rotate the model
    transform.rotation[1] = t * 0.5;
    transform.rotation[0] = t * 0.3;

    frame.clear_color(0.0, 0.0, 0.0, 1.0);

    let uniforms = uniform! {
        model: transform.get_model().0,
        view: camera.get_view(),
        proj: camera.get_proj(),

        texture: sampler,
        ambient: 0.3f32,
        light: utils::math::normalize([-1.0, 0.4, 0.9f32]),
    };

    if let Err(e) = frame.draw(&model.vertices, &model.indices, &render_resource.program, &uniforms, &Default::default())
        { eprintln!("Error drawing frame: {}", e.to_string()); return; }
    if let Err(e) = frame.finish() { eprintln!("Error finishing frame: {}", e.to_string()); }
}
