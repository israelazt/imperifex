//! The window: winit's events go to the application and the interface, and what
//! they answer goes back to the real window and the renderer. No decisions.

mod translate;

use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use imperifex_app::window::{Core, WindowCommand};
use imperifex_domain::i18n::{text, Language, Text};
use imperifex_domain::mouse::Mouse;
use imperifex_domain::ports::{Clock, Renderer, Wake};
use imperifex_domain::settings::Settings;
use imperifex_domain::view::Pointer;
use imperifex_domain::{log_error, log_info, log_warn};
use imperifex_platform::{D3dRenderer, DirectWriteText, SystemClock, WindowsAppearance};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// What other threads tell the window.
#[derive(Debug, Clone, Copy)]
enum Told {
    ThemeChanged,
}

/// Opens the window and stays until it is closed.
pub fn run(settings: Settings, language: Language) -> Result<(), String> {
    let events = EventLoop::<Told>::with_user_event()
        .build()
        .map_err(|e| format!("no event loop: {e}"))?;
    events.set_control_flow(ControlFlow::Wait);
    let letters = Rc::new(DirectWriteText::new(&settings.font.family)?);
    if !letters.found_grid_font() {
        let family = &settings.font.family;
        log_warn!("{} ({family})", text(language, Text::FontNotFound));
    }
    let core = Core::new(settings, language, Box::new(WindowsAppearance));
    let proxy = events.create_proxy();
    let changed: Wake = Arc::new(move || {
        // It only fails once the loop is over: nobody is left to tell.
        let _ = proxy.send_event(Told::ThemeChanged);
    });
    if let Err(error) = core.watch_system(changed) {
        log_warn!("changes of the system theme will not be seen: {error}");
    }
    let mut app = App {
        core,
        mouse: Mouse::default(),
        clock: SystemClock::new(),
        letters,
        shown: None,
        frames: 0,
    };
    let result = events
        .run_app(&mut app)
        .map_err(|e| format!("the event loop stopped: {e}"));
    log_info!("window closed after {} frames drawn", app.frames);
    result
}

struct App {
    core: Core,
    mouse: Mouse,
    clock: SystemClock,
    /// Measures the texts for the interface and pictures them for the renderer.
    letters: Rc<DirectWriteText>,
    shown: Option<Shown>,
    /// How many frames were drawn, to see that an idle window draws none.
    frames: u64,
}

/// The real window and what draws on it.
struct Shown {
    window: Window,
    renderer: Option<D3dRenderer>,
    /// The last renderer failed: another failure does not ask for one more
    /// try, which would go on for ever.
    failed: bool,
}

impl Shown {
    /// Drops the renderer after an error, and asks once for a new one.
    fn lost(&mut self, error: &str) {
        log_error!("the window could not be drawn: {error}");
        self.renderer = None;
        if !self.failed {
            self.failed = true;
            self.window.request_redraw();
        }
    }

    fn renderer(&mut self, letters: &Rc<DirectWriteText>) -> Result<&mut D3dRenderer, String> {
        if self.renderer.is_none() {
            let size = self.window.inner_size();
            let handle = translate::handle(&self.window)?;
            let made = D3dRenderer::new(handle, size.width, size.height, Rc::clone(letters))?;
            self.renderer = Some(made);
        }
        self.renderer.as_mut().ok_or_else(|| "no renderer".into())
    }
}

impl App {
    /// Draws what there is now and shows it.
    fn draw(&mut self) {
        let Some(shown) = &mut self.shown else { return };
        let model = self.core.model();
        if model.window.0 == 0 || model.window.1 == 0 {
            return;
        }
        let pointer = self.mouse.hover(self.clock.monotonic());
        let list = imperifex_ui::frame(&model, &pointer, self.letters.as_ref()).draw;
        let renderer = shown.renderer(&self.letters);
        match renderer.and_then(|renderer| renderer.draw(&list)) {
            Ok(()) => {
                shown.failed = false;
                self.frames += 1;
                self.core.was_shown(list);
            }
            Err(error) => shown.lost(&error),
        }
    }

    /// Does what the mouse asks, and asks for a redraw if what is seen changed.
    fn mouse_did(&mut self, event_loop: &ActiveEventLoop, pointer: Pointer) {
        let frame = imperifex_ui::frame(&self.core.model(), &pointer, self.letters.as_ref());
        if let Some(shown) = &self.shown {
            shown.window.set_cursor(translate::cursor(frame.cursor));
        }
        for action in frame.actions {
            for command in self.core.apply(action) {
                self.obey(event_loop, command);
            }
        }
        self.redraw_if_changed();
    }

    fn redraw_if_changed(&self) {
        let Some(shown) = &self.shown else { return };
        let pointer = self.mouse.hover(self.clock.monotonic());
        let list = imperifex_ui::frame(&self.core.model(), &pointer, self.letters.as_ref()).draw;
        if self.core.is_news(&list) {
            shown.window.request_redraw();
        }
    }

    fn obey(&mut self, event_loop: &ActiveEventLoop, command: WindowCommand) {
        let Some(shown) = &self.shown else { return };
        let window = &shown.window;
        let moved = match command {
            WindowCommand::Minimize => {
                // The button is not under the mouse when the window comes back.
                self.mouse.left();
                window.set_minimized(true);
                Ok(())
            }
            WindowCommand::SetMaximized(maximized) => {
                window.set_maximized(maximized);
                Ok(())
            }
            WindowCommand::Close => {
                event_loop.exit();
                Ok(())
            }
            WindowCommand::Drag => window.drag_window(),
            WindowCommand::Resize(edge) => window.drag_resize_window(translate::direction(edge)),
            WindowCommand::ShowSystemMenu => {
                if let Some((x, y)) = self.mouse.position() {
                    window.show_window_menu(PhysicalPosition::new(x, y));
                }
                Ok(())
            }
            WindowCommand::Redraw => {
                window.request_redraw();
                Ok(())
            }
        };
        if let Err(error) = moved {
            log_warn!("the window did not obey {command:?}: {error}");
        }
    }

    /// The window's size changed, or whether it is maximized.
    fn resized(&mut self) {
        let Some(shown) = &mut self.shown else { return };
        let size = shown.window.inner_size();
        self.core
            .resized((size.width, size.height), shown.window.is_maximized());
        if let Some(renderer) = &mut shown.renderer {
            if let Err(error) = renderer.resize(size.width, size.height) {
                shown.lost(&error);
                return;
            }
        }
        shown.window.request_redraw();
    }

    fn mouse_event(&mut self, event: &WindowEvent) -> Option<Pointer> {
        Some(match *event {
            WindowEvent::CursorMoved { position, .. } => {
                let at = self.clock.monotonic();
                self.mouse.moved(translate::point(position), at)
            }
            WindowEvent::CursorLeft { .. } | WindowEvent::Focused(false) => self.mouse.left(),
            WindowEvent::MouseInput { state, button, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => {
                    self.mouse.pressed(self.clock.monotonic())
                }
                (MouseButton::Left, ElementState::Released) => self.mouse.released(),
                (MouseButton::Right, ElementState::Pressed) => self.mouse.right_pressed(),
                _ => return None,
            },
            _ => return None,
        })
    }
}

impl ApplicationHandler<Told> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.shown.is_some() {
            return;
        }
        let attributes = translate::attributes(self.core.model().title);
        let window = match event_loop.create_window(attributes) {
            Ok(window) => window,
            Err(error) => {
                log_error!("the window could not be created: {error}");
                event_loop.exit();
                return;
            }
        };
        // A scale of a screen is far inside what an `f32` holds.
        self.core.set_scale(window.scale_factor() as f32);
        self.shown = Some(Shown {
            window,
            renderer: None,
            failed: false,
        });
        self.resized();
        // Drawn before it is seen, so that it never shows empty.
        self.draw();
        if let Some(shown) = &self.shown {
            shown.window.set_visible(true);
        }
    }

    fn new_events(&mut self, _: &ActiveEventLoop, cause: StartCause) {
        // The only wait with an end is for the mouse to rest.
        if matches!(cause, StartCause::ResumeTimeReached { .. }) {
            self.redraw_if_changed();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let rest = self.mouse.rests_in(self.clock.monotonic());
        event_loop.set_control_flow(match rest {
            Some(left) => ControlFlow::WaitUntil(Instant::now() + left),
            None => ControlFlow::Wait,
        });
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, told: Told) {
        match told {
            Told::ThemeChanged => {
                for command in self.core.system_theme_changed() {
                    self.obey(event_loop, command);
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let WindowEvent::Focused(focused) = event {
            self.core.set_focused(focused);
        }
        if let Some(pointer) = self.mouse_event(&event) {
            self.mouse_did(event_loop, pointer);
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(_) => self.resized(),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.core.set_scale(scale_factor as f32);
                let renderer = self.shown.as_mut().and_then(|s| s.renderer.as_mut());
                if let Some(renderer) = renderer {
                    renderer.forget_glyphs();
                }
                self.redraw_if_changed();
            }
            WindowEvent::Focused(true) => self.redraw_if_changed(),
            _ => {}
        }
    }
}
