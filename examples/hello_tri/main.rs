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
}

impl App {
    pub fn new() -> Self {
        Self {
            renderer: None,
            window: None,
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
            WindowEvent::RedrawRequested => {
                if let Some(_window) = &mut self.window {
                    if let Some(renderer) = &self.renderer {
                        renderer.run();
                    }
                }
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == Key::Named(NamedKey::Escape) {
                    log::info!("Window closing");
                    event_loop.exit();
                }
            }
            _ => {}
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
