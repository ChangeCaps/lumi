use std::time::Instant;

use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform::{Platform, PlatformDescriptor};
use futures_lite::future;

use lumi::{prelude::*, *};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub fn framework(
    mut world: World,
    mut f: impl FnMut(Event<()>, &mut Renderer, &mut World, &egui::Context) + 'static,
) -> ! {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .init();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    let instance = Instance::new(Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = future::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();
    let (device, queue) =
        future::block_on(adapter.request_device(&DeviceDescriptor::default(), None)).unwrap();

    let mut configuration = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: TextureFormat::Bgra8UnormSrgb,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: PresentMode::Fifo,
    };
    let mut resized = true;

    let mut renderer = Renderer::new(&device);

    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: window.inner_size().width,
        physical_height: window.inner_size().height,
        scale_factor: window.scale_factor(),
        font_definitions: Default::default(),
        style: Default::default(),
    });

    let mut egui_render_pass = egui_wgpu_backend::RenderPass::new(&device, configuration.format, 1);
    let start_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        platform.handle_event(&event);

        match event {
            Event::RedrawRequested(_) => {
                if resized {
                    surface.configure(&device, &configuration);
                    resized = false;
                }
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    configuration.width = new_size.width;
                    configuration.height = new_size.height;
                    resized = true;
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    configuration.width = new_inner_size.width;
                    configuration.height = new_inner_size.height;
                    resized = true;
                }
                _ => {}
            },
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }

        let is_redraw_requested = match event {
            Event::RedrawRequested(_) => true,
            _ => false,
        };

        if is_redraw_requested {
            platform.update_time(start_time.elapsed().as_secs_f64());
            platform.begin_frame();

            f(event, &mut renderer, &mut world, &platform.context());

            let target = surface.get_current_texture().unwrap();
            let target_view = target.texture.create_view(&Default::default());
            let render_target = RenderTarget {
                view: &target_view,
                width: configuration.width,
                height: configuration.height,
            };

            renderer.render(&device, &queue, &world, &render_target);

            let full_output = platform.end_frame(Some(&window));
            let paint_jobs = platform.context().tessellate(full_output.shapes);

            let mut encoder = device.create_command_encoder(&Default::default());

            let screen_descriptor = ScreenDescriptor {
                physical_width: configuration.width,
                physical_height: configuration.height,
                scale_factor: window.scale_factor() as f32,
            };
            let tdelta = full_output.textures_delta;
            egui_render_pass
                .add_textures(&device, &queue, &tdelta)
                .unwrap();
            egui_render_pass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);
            egui_render_pass
                .execute(
                    &mut encoder,
                    &target_view,
                    &paint_jobs,
                    &screen_descriptor,
                    None,
                )
                .unwrap();

            queue.submit(std::iter::once(encoder.finish()));

            target.present();

            egui_render_pass.remove_textures(tdelta).unwrap();
        }
    });
}
