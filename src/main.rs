use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use epi::App;
use winit::{event_loop, event_loop::{EventLoop}, window, event::{self, WindowEvent}};

pub mod tear_app;

enum Event {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct AppRepaintSignal {
    crit_sec: std::sync::Mutex<winit::event_loop::EventLoopProxy<Event>>
}

impl epi::backend::RepaintSignal for AppRepaintSignal {
    fn request_repaint(&self) {
        self.crit_sec
            .lock()
            .unwrap()
            .send_event(Event::RequestRedraw).ok();
    }
}

fn main() {
    let event_loop = EventLoop::with_user_event();
    let mut builder = window::WindowBuilder::new();
    builder = builder.with_title("tear-grep");
    let window = builder.build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = {
        let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
        let request = wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface));
        pollster::block_on(request)
            .expect("Failed to construct gpu adapter!")
    };

    {
        let adapter_info = adapter.get_info();
        println!("GPU: {} ({:?})", adapter_info.name, adapter_info.backend);    
    }
    
    let (device, queue) = {
        let request = adapter.request_device(
            &wgpu::DeviceDescriptor{
                label: None,
                features: Default::default(),
                limits: Default::default(),
            },
            None
        );

        pollster::block_on(request)
            .expect("Failed to construct GPU device or queue!")
    };

    let surface_size = window.inner_size();
    let surface_fmt = surface.get_preferred_format(&adapter).unwrap();
    let mut surface_cfg = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_fmt,
        width: surface_size.width,
        height: surface_size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(
        &device,
        &surface_cfg);

    let repaint_signal = std::sync::Arc::new(
        AppRepaintSignal {
            crit_sec: std::sync::Mutex::new(
                event_loop.create_proxy()
            )
        });

    let max_texture_side = 4096;
    let mut state = egui_winit::State::new(max_texture_side, &window);
    let context = egui::Context::default();

    let mut egui_renderpass = RenderPass::new(&device, surface_fmt, 1);

    // let mut demo_app = egui_demo_lib::WrapApp::default();
    let mut demo_app = tear_app::App::new();

    let mut start_time = std::time::Instant::now();
    let mut last_frame_time : Option<f32> = None;

    event_loop.run(move |event, _window_target, control_flow| {
        if start_time.elapsed().as_secs() > 3600 {
            // Reset our timer after an hour to
            // avoid generating inaccurate dts.
            start_time = std::time::Instant::now();
        }

        match event {
            event::Event::RedrawRequested(..) => {
                let backbuffer = match surface.get_current_texture() {
                    Ok(f) => f,
                    // On error, skip this iteration of the event_loop and just return.
                    Err(wgpu::SurfaceError::Outdated) => {
                        // Can happen when app is minimzed, but is ok i think
                        return;
                    },
                    Err(e) => {
                        eprintln!("Dropped frame with error {}", e);
                        surface.configure(&device, &surface_cfg);
                        surface
                            .get_current_texture()
                            .expect("Failed to get next backbuffer")
                    }
                };

                let rt_view = backbuffer
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let render_start_time = std::time::Instant::now();

                let input = state.take_egui_input(&window);
                context.begin_frame(input);

                let app_out = epi::backend::AppOutput::default();
                
                let frame = epi::Frame::new(epi::backend::FrameData {
                    info: epi::IntegrationInfo {
                        name: "egui_winit_wgpu",
                        web_info: None,
                        prefer_dark_mode: None,
                        cpu_usage: last_frame_time,
                        native_pixels_per_point: Some(window.scale_factor() as _),
                    },
                    output: app_out,
                    repaint_signal: repaint_signal.clone(),
                });

                demo_app.update(&context, &frame);

                let egui_out = context.end_frame();
                let paint_jobs = context.tessellate(egui_out.shapes);
                
                last_frame_time = {
                    let passed_time = (std::time::Instant::now() - render_start_time).as_secs_f64();
                    Some(passed_time as f32)
                };

                let mut cmd_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("egui_commands")
                });

                let view_descriptor = ScreenDescriptor {
                    physical_width: surface_cfg.width,
                    physical_height: surface_cfg.height,
                    scale_factor: window.scale_factor() as f32
                };

                egui_renderpass.add_textures(&device, &queue, &egui_out.textures_delta).unwrap();
                egui_renderpass.remove_textures(egui_out.textures_delta).unwrap();
                egui_renderpass.update_buffers(&device, &queue, &paint_jobs, &view_descriptor);

                egui_renderpass.execute(
                    &mut cmd_encoder,
                    &rt_view,
                    &paint_jobs,
                    &view_descriptor,
                    None
                ).unwrap();

                queue.submit(std::iter::once(cmd_encoder.finish()));
                backbuffer.present();
            },
            winit::event::Event::MainEventsCleared | winit::event::Event::UserEvent(Event::RequestRedraw) => {
                window.request_redraw();
            }
            event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                    if size.width > 0 && size.height > 0 {
                        surface_cfg.width = size.width;
                        surface_cfg.height = size.height;
                        surface.configure(&device, &surface_cfg);
                    }
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = event_loop::ControlFlow::Exit;
                }
                event => {
                    // Pass the winit events to the platform integration.
                    state.on_event(&context, &event);
                }
            },
            _ => (),
        }
    });
}
