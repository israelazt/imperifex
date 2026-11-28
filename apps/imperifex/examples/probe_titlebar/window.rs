//! The probe's window: what each mouse action asks of winit, and printing what
//! comes back.

#[path = "paint.rs"]
mod paint;
#[path = "zones.rs"]
mod zones;

use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{CursorIcon, Window, WindowId};

use zones::{zone, Zone};

const DOUBLE_CLICK: Duration = Duration::from_millis(400);

pub fn run() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut Probe {
        window: None,
        opened: Instant::now(),
        mouse: PhysicalPosition::default(),
        last_bar_press: None,
        last_state: String::new(),
    })
}

struct Probe {
    window: Option<Window>,
    opened: Instant,
    mouse: PhysicalPosition<f64>,
    last_bar_press: Option<Instant>,
    last_state: String,
}

/// Where the window is and how it is, in physical pixels.
fn state(window: &Window) -> String {
    let size = window.inner_size();
    let at = window
        .outer_position()
        .map_or("?".into(), |p| format!("{},{}", p.x, p.y));
    let inside = window
        .inner_position()
        .map_or("?".into(), |p| format!("{},{}", p.x, p.y));
    let monitor = window.current_monitor().map_or("?".into(), |m| {
        let name = m.name().unwrap_or_default();
        format!("{name} escala {}", m.scale_factor())
    });
    format!(
        "pos {at} interior {inside} tamaño {}x{} maximizada {} minimizada {:?} monitor {monitor}",
        size.width,
        size.height,
        window.is_maximized(),
        window.is_minimized(),
    )
}

impl Probe {
    fn say(&self, what: &str) {
        println!("{:8.3} s  {what}", self.opened.elapsed().as_secs_f64());
    }

    /// Prints the window's state if it is not what was printed last.
    fn report(&mut self, why: &str) {
        let Some(window) = &self.window else { return };
        let now = state(window);
        if now != self.last_state {
            self.say(&format!("{why}: {now}"));
            self.last_state = now;
        }
    }

    fn zone_under_mouse(&self, window: &Window) -> Zone {
        let (size, scale) = (window.inner_size(), window.scale_factor());
        zone(self.mouse, size, scale, window.is_maximized())
    }

    fn press(&mut self, button: MouseButton) {
        let Some(window) = &self.window else { return };
        let zone = self.zone_under_mouse(window);
        let result = match (button, zone) {
            (MouseButton::Left, Zone::Edge(direction)) => {
                format!("{:?}", window.drag_resize_window(direction))
            }
            (MouseButton::Left, Zone::MaximizeSlot) => {
                window.set_maximized(!window.is_maximized());
                "alternar maximizada".into()
            }
            (MouseButton::Left, Zone::Bar) => {
                let now = Instant::now();
                let double = self
                    .last_bar_press
                    .is_some_and(|at| now - at <= DOUBLE_CLICK);
                self.last_bar_press = (!double).then_some(now);
                if double {
                    window.set_maximized(!window.is_maximized());
                    "doble clic: alternar maximizada".into()
                } else {
                    format!("arrastrar: {:?}", window.drag_window())
                }
            }
            (MouseButton::Right, Zone::Bar | Zone::MaximizeSlot) => {
                window.show_window_menu(self.mouse);
                "menú de la ventana".into()
            }
            _ => return,
        };
        self.say(&format!("{button:?} en {zone:?}: {result}"));
    }

    fn moved(&mut self, to: PhysicalPosition<f64>) {
        self.mouse = to;
        let Some(window) = &self.window else { return };
        window.set_cursor(match self.zone_under_mouse(window) {
            Zone::Edge(direction) => CursorIcon::from(direction),
            _ => CursorIcon::Default,
        });
    }
}

impl ApplicationHandler for Probe {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("probe_titlebar (Esc cierra)")
            .with_inner_size(LogicalSize::new(900.0, 560.0))
            .with_decorations(false)
            .with_undecorated_shadow(true);
        match event_loop.create_window(attributes) {
            Ok(window) => self.window = Some(window),
            Err(error) => {
                self.say(&format!("no se pudo crear la ventana: {error}"));
                event_loop.exit();
            }
        }
        self.report("abierta");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.say("pidieron cerrar");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    paint::paint(window);
                }
            }
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                self.report("cambió de tamaño");
            }
            WindowEvent::Moved(_) => self.report("se movió"),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.say(&format!("ScaleFactorChanged: {scale_factor}"));
            }
            WindowEvent::CursorMoved { position, .. } => self.moved(position),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => self.press(button),
            _ => {}
        }
    }
}
