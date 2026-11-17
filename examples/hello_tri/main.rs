use std::time::Instant;

use renderer::renderer::Renderer;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::Window,
};

struct App {
    renderer: Option<Renderer>, // Comes first to destroy first on drop
    window: Option<Window>,

    frames: u64,
    last: std::time::Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            renderer: None,
            window: None,
            frames: 0,
            last: Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("Test"))
            .unwrap();
        let window_size = window.inner_size();

        match Renderer::new(&window, window_size.width, window_size.height) {
            Ok(renderer) => {
                self.window = Some(window);
                self.renderer = Some(renderer);
            }
            Err(e) => {
                eprintln!("Error making renderer:\n\t{}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Window closing");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size.width, new_size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                self.frames += 1;
                if let Some(renderer) = &mut self.renderer {
                    if let Err(e) = renderer.render() {
                        eprintln!("{e}");
                        event_loop.exit();
                    }
                }

                if self.last.elapsed() >= std::time::Duration::from_secs(1) {
                    log::info!("{}fps", self.frames);
                    self.frames = 0;
                    self.last = Instant::now();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == Key::Named(NamedKey::Escape) {
                    // log::info!("Window closing");
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &mut self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    match event_loop.run_app(&mut app) {
        Err(e) => eprintln!("{}", e),
        _ => {}
    }
}
