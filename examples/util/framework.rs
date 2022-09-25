use futures_lite::future;
use lumi::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub fn framework(mut f: impl FnMut(Event<()>, &mut Renderer, &Surface) + 'static) -> ! {
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
    let mut resized = false;

    let device = SharedDevice::new(device);
    let queue = SharedQueue::new(queue);

    let mut renderer = Renderer::new(device, queue);
    renderer.resize(configuration.width, configuration.height);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                if resized {
                    surface.configure(&renderer.device, &configuration);
                    resized = false;
                }
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    configuration.width = new_size.width;
                    configuration.height = new_size.height;
                    resized = true;
                    renderer.resize(new_size.width, new_size.height);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    configuration.width = new_inner_size.width;
                    configuration.height = new_inner_size.height;
                    resized = true;
                    renderer.resize(new_inner_size.width, new_inner_size.height);
                }
                _ => (),
            },
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }

        f(event, &mut renderer, &surface);
    });
}
