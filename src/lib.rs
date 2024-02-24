#![deny(elided_lifetimes_in_paths)]
use std::{borrow::Cow, sync::Arc};

#[allow(unused)]
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::filter::EnvFilter;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    init_logging();

    #[cfg(not(target_arch = "wasm32"))]
    let builder = WindowBuilder::new();
    #[cfg(target_arch = "wasm32")]
    let builder = {
        let mut builder = WindowBuilder::new();
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
        builder.with_canvas(Some(canvas))
    };

    let event_loop = EventLoop::new().unwrap();
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
        surface = Some(instance.create_surface(window.clone()).unwrap());
    }

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: surface.as_ref(),
        })
        .await
        .unwrap();

    info!("{adapter:?}");

    let (device, _queue) = adapter
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

    #[derive(Debug)]
    struct Render {
        shader: wgpu::ShaderModule,
        pipeline: wgpu::RenderPipeline,
    }

    let mut render: Option<Render> = None;

    let _ = (event_loop_function)(
        event_loop,
        move |event: Event<()>, _target: &EventLoopWindowTarget<()>| match event {
            Event::Resumed => {
                info!("resumed");
                if surface.is_none() {
                    let new_surface = instance.create_surface(window.clone()).unwrap();
                    info!("{new_surface:?}");
                    let swapchain_capabilities = new_surface.get_capabilities(&adapter);
                    info!("{swapchain_capabilities:?}");
                    let swapchain_format = swapchain_capabilities.formats[0];

                    let mut size = window.inner_size();
                    size.width = size.width.max(1);
                    size.height = size.height.max(1);

                    let config = new_surface
                        .get_default_config(&adapter, size.width, size.height)
                        .unwrap();

                    new_surface.configure(&device, &config);

                    //*surface = Some(instance.create_surface(window.clone()).unwrap());
                    surface = Some(new_surface);
                    debug!("made surface: {surface:?}");

                    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                            "shader.wgsl"
                        ))),
                    });

                    let pipeline_layout =
                        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[],
                            push_constant_ranges: &[],
                        });

                    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: "vs_main",
                            buffers: &[],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_main",
                            targets: &[Some(swapchain_format.into())],
                        }),
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                        multiview: None,
                    });

                    render = Some(Render { shader, pipeline });
                    info!("{render:?}");
                }
            }
            Event::Suspended => {
                info!("suspended");
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(_size) => {
                    info!("resized: {_size:?}")
                }
                WindowEvent::KeyboardInput { .. } => {
                    info!("keyboard_input")
                }
                WindowEvent::CloseRequested => {
                    info!("close_requested")
                }
                WindowEvent::RedrawRequested => {
                    if render.is_some() {
                        info!("{:?}", render.as_mut().unwrap().shader);
                    }

                    info!("redraw_requested")
                }
                _ => {}
            },
            _ => {}
        },
    );
}

fn init_logging() {
    #[cfg(target_arch = "wasm32")]
    {
        /*let base_level = log::LevelFilter::Info;
        let wgpu_level = log::LevelFilter::Error;
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{} \"{}\":{} {}",
                    record.level(),
                    record.file().unwrap_or(""),
                    record.line().unwrap_or(0),
                    message
                ))
            })
            .level(base_level)
            .level_for("wgpu_core", wgpu_level)
            .level_for("wgpu_hal", wgpu_level)
            .level_for("naga", wgpu_level)
            .chain(fern::Output::call(console_log::log))
            .apply()
            .unwrap();
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));*/
        //tracing_wasm::set_as_global_default();
    }

    let subscriber = tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_source_location(true)
                .without_time(),
                //.compact(),
        )
        .with_env_filter(EnvFilter::from_default_env());
    #[cfg(not(target_arch = "wasm32"))]
    subscriber.init();
    #[cfg(target_arch = "wasm32")]
    tracing_wasm::set_as_global_default();

    info!("Logging initialized");
}
