//! One frame: the interface is laid out, the shell learns the room it has and
//! the renderer draws both. No decisions.

use imperifex_domain::log_error;
use imperifex_domain::ports::{Clock, Renderer};
use imperifex_domain::view::Pointer;
use imperifex_platform::{ConPtyLauncher, VteParser};
use imperifex_ui::Frame;

use super::App;
use imperifex_app::window::Start;

impl App {
    pub(super) fn pointer(&self) -> Pointer {
        self.mouse.hover(self.clock.monotonic())
    }

    pub(super) fn frame(&self, pointer: &Pointer) -> Frame {
        imperifex_ui::frame(&self.core.model(), pointer, self.letters.as_ref())
    }

    /// The frame to draw, once the shell knows where its grid goes. The first
    /// one opens it, and what that changes is laid out again.
    fn laid_out(&mut self) -> Frame {
        let frame = self.frame(&self.pointer());
        let scale = self.core.model().scale;
        match self.letters.cell(self.core.font_points(), scale) {
            Ok(cell) => {
                let now = self.clock.monotonic();
                let (full, seen) = (frame.terminal_area, frame.output_area);
                self.core.fit(full, seen, cell, now);
            }
            Err(error) => log_error!("the grid could not be measured: {error}"),
        }
        let start = Start {
            launcher: &ConPtyLauncher,
            new_parser: &|| Box::new(VteParser::new()),
            wake: self.output_arrived.clone(),
        };
        if self.core.open_shell(&start) {
            return self.frame(&self.pointer());
        }
        frame
    }

    /// Draws what there is now and shows it.
    pub(super) fn draw(&mut self) {
        let (width, height) = self.core.model().window;
        if self.shown.is_none() || width == 0 || height == 0 {
            return;
        }
        let list = self.laid_out().draw;
        let damage = self.core.take_damage();
        let Some(shown) = &mut self.shown else { return };
        let terminal = self.core.picture(&damage);
        let renderer = shown.renderer(&self.letters);
        match renderer.and_then(|renderer| renderer.draw(&list, terminal.as_ref())) {
            Ok(()) => {
                shown.failed = false;
                self.frames += 1;
                self.core.was_shown(list);
            }
            Err(error) => {
                shown.lost(&error);
                self.core.not_shown();
            }
        }
    }

    /// Asks for a frame if what the interface shows changed.
    pub(super) fn redraw_if_changed(&self) {
        let Some(shown) = &self.shown else { return };
        let list = self.frame(&self.pointer()).draw;
        if self.core.is_news(&list) {
            shown.window.request_redraw();
        }
    }
}
