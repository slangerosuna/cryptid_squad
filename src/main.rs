use tokio::main;
use glium::*;

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

    let mut frame = display.draw();

    frame.clear_color(0.0, 0.0, 1.0, 1.0);

    frame.finish()
        .expect("Failed to swap buffers");
}
