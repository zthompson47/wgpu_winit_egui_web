use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() -> Result<(), impl std::error::Error> {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let instance = wgpu::Instance::default();
    let surface = instance.request_adapter(&wgpu::RequestAdapterOptions {
    }).await

    event_loop.run(move |event, window| {
        println!("{event:?}");
    })
}
