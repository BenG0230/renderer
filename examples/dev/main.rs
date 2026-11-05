use anyhow::{Result, anyhow};
use renderer::renderer::Renderer;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::Window,
};

struct App {
    window: Option<Window>,
}

impl App {
    pub fn new() -> Self {
        Self { window: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("Test"))
            .unwrap();

        self.window = Some(window);
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
                    //Redraw am i right
                }
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

fn main() -> Result<()> {
    env_logger::init();
    // let event_loop = EventLoop::new().unwrap();
    // let mut app = App::new();
    // event_loop.run_app(&mut app)?;

    match Renderer::new() {
        Ok(rend) => rend.run(),
        Err(e) => {
            log::error!("Failed to run application: {}", e)
        }
    }

    Ok(())
}
