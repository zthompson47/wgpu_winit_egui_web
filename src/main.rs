#![deny(elided_lifetimes_in_paths)]
use std::sync::Arc;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};

#[pollster::main]
async fn main() -> Result<(), impl std::error::Error> {
    let event_loop = EventLoop::new().unwrap();

    let builder = WindowBuilder::new();
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        builder = builder.with_canvas(Some(canvas));
    }
    let window = Arc::new(builder.build(&event_loop).unwrap());

    let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
    let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
    let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        dx12_shader_compiler,
        gles_minor_version,
    });

    let mut surface = None;
    #[cfg(target_arch = "wasm32")]
    {
        surface = Some(instance.create_surface(window).unwrap());
    }

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: surface.as_ref(),
        })
        .await
        .unwrap();

    let (_device, _queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            },
            None,
        )
        .await
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    let event_loop_function = {
        use winit::platform::web::EventLoopExtWebSys;
        EventLoop::spawn
    };
    #[cfg(not(target_arch = "wasm32"))]
    let event_loop_function = EventLoop::run;

    (event_loop_function)(
        event_loop,
        move |event: Event<()>, _target: &EventLoopWindowTarget<()>| match event {
            Event::Resumed => {
                if surface.is_none() {
                    surface = Some(instance.create_surface(window.clone()).unwrap());
                }
            }
            Event::Suspended => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(_size) => {}
                WindowEvent::KeyboardInput { .. } => {}
                WindowEvent::CloseRequested => {}
                WindowEvent::RedrawRequested => {}
                _ => {}
            },
            _ => {}
        },
    )
}
