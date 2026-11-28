//! The shell of the window: when it is opened, when its output is read and what
//! of it is drawn, and where.

use std::time::Duration;

use imperifex_domain::draw::Rect;
use imperifex_domain::font::{pixels_of_points, CellSize};
use imperifex_domain::grid::Damage;
use imperifex_domain::log_warn;
use imperifex_domain::picture::TerminalPicture;
use imperifex_domain::ports::{Clock, PtyLauncher, SequenceParser, Wake};
use imperifex_domain::shell_state::ShellState;
use imperifex_domain::theme::palette;
use imperifex_domain::view::Point;

use super::{redraw_if, Core, WindowCommand};
use crate::shell_tab::{Opening, ShellTab};

/// What opening the window's shell needs from the program.
pub struct Start<'a> {
    pub launcher: &'a dyn PtyLauncher,
    pub new_parser: &'a dyn Fn() -> Box<dyn SequenceParser>,
    /// Called when there is something to read: then [`Core::read_output`] is
    /// to be asked.
    pub wake: Wake,
}

#[derive(Default)]
enum Shell {
    #[default]
    NotOpened,
    Open(Box<ShellTab>),
    /// No shell started; why the last one did not.
    Failed(String),
}

#[derive(Debug, Clone, Copy)]
struct Place {
    /// All the room of the output: the console is as large as it.
    full: Rect,
    /// What of it is left for the grid, less with the editor under it.
    seen: Rect,
    cell: CellSize,
}

/// What of the terminal is drawn, and where.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Layout {
    pub area: Rect,
    /// The row of the screen at the top of the area: negative, of the history.
    pub first: isize,
    /// Rows at the top of the screen out of sight though the view is not back.
    hidden: usize,
}

#[derive(Default)]
pub(super) struct Output {
    shell: Shell,
    /// Where the grid goes and the size of its cells, as last drawn.
    place: Option<Place>,
    /// Output asked for a frame that is not on the window yet. Until it is, no
    /// more is read: a program that does not stop writing would leave the
    /// window no turn to draw.
    awaiting_frame: bool,
    /// There may be output left to read.
    unread: bool,
}

impl Output {
    pub fn tab(&self) -> Option<&ShellTab> {
        match &self.shell {
            Shell::Open(tab) => Some(tab),
            _ => None,
        }
    }

    pub(super) fn tab_mut(&mut self) -> Option<&mut ShellTab> {
        match &mut self.shell {
            Shell::Open(tab) => Some(tab),
            _ => None,
        }
    }

    pub fn failure(&self) -> Option<&str> {
        match &self.shell {
            Shell::Failed(why) => Some(why),
            _ => None,
        }
    }

    pub fn was_shown(&mut self) {
        self.awaiting_frame = false;
    }

    /// The cell of the grid at a place of the window, or the closest one.
    fn cell_at(&self, (x, y): Point) -> Option<(u16, u16)> {
        let Place {
            full: area, cell, ..
        } = self.place?;
        let (columns, rows) = self.size()?;
        let along = |at: i32, start: i32, side: u32, count: u16| {
            let from = i64::from(at) - i64::from(start);
            let whole = from / i64::from(side.max(1));
            u16::try_from(whole.clamp(0, i64::from(count) - 1)).unwrap_or(0)
        };
        Some((
            along(x, area.x, cell.width, columns),
            along(y, area.y, cell.height, rows),
        ))
    }

    /// The size of the grid's cells, as last drawn.
    pub fn cell(&self) -> Option<CellSize> {
        Some(self.place?.cell)
    }

    /// Columns and rows that fit where the grid goes.
    fn size(&self) -> Option<(u16, u16)> {
        let Place { full, cell, .. } = self.place?;
        Some(cell.fit(full.width, full.height))
    }
}

impl Core {
    /// The size of the grid's letters in points, to measure its cell.
    pub fn font_points(&self) -> f32 {
        self.settings.font.size
    }

    /// The output has all of `full`, and of it the grid now goes in `seen`,
    /// with cells of `cell`.
    pub fn fit(&mut self, full: Rect, seen: Rect, cell: CellSize, now: Duration) {
        self.output.place = Some(Place { full, seen, cell });
        let size = self.output.size();
        if let (Some(size), Some(tab)) = (size, self.output.tab_mut()) {
            tab.room_for(size, now);
        }
    }

    /// Opens the shell with the size that fits. Only the first call after
    /// [`Core::fit`] tries. `true` if there is something new to show.
    pub fn open_shell(&mut self, start: &Start<'_>) -> bool {
        let Some(size) = self.output.size() else {
            return false;
        };
        if !matches!(self.output.shell, Shell::NotOpened) {
            return false;
        }
        let opened = ShellTab::open(&Opening {
            launcher: start.launcher,
            new_parser: start.new_parser,
            chosen: self.settings.shell.as_deref(),
            size,
            wake: start.wake.clone(),
            integrate: self.settings.shell_integration,
        });
        self.output.shell = match opened {
            Ok(tab) => {
                self.typing.start(tab.line());
                Shell::Open(Box::new(tab))
            }
            Err(why) => Shell::Failed(why),
        };
        true
    }

    /// With nothing seen, as when minimized, no frame is waited for.
    fn is_visible(&self) -> bool {
        self.size.0 > 0 && self.size.1 > 0
    }

    fn must_wait_for_the_frame(&self) -> bool {
        self.output.awaiting_frame && self.is_visible()
    }

    /// Reads what the shell wrote, unless a frame is still to be drawn: then it
    /// stays for later, and [`Core::has_unread`] says so after the frame. A
    /// shell that ends closes the window.
    pub fn read_output(&mut self, clock: &dyn Clock) -> Vec<WindowCommand> {
        if self.must_wait_for_the_frame() {
            self.output.unread = true;
            return Vec::new();
        }
        let Some(tab) = self.output.tab_mut() else {
            return Vec::new();
        };
        let read = tab.read(clock);
        self.output.unread = read.more;
        if read.ended {
            self.output.unread = false;
            return vec![WindowCommand::Close];
        }
        if !read.redraw {
            return Vec::new();
        }
        self.output.awaiting_frame = true;
        vec![WindowCommand::Redraw]
    }

    /// Whether [`Core::read_output`] is to be asked again without waiting for
    /// the shell.
    pub fn has_unread(&self) -> bool {
        self.output.unread && !self.must_wait_for_the_frame()
    }

    /// The frame could not be drawn: the output is not held back for it.
    pub fn not_shown(&mut self) {
        self.output.awaiting_frame = false;
    }

    /// Gives the shell its new size once it has stayed the same long enough.
    pub fn settle_size(&mut self, now: Duration) -> Vec<WindowCommand> {
        let changed = self
            .output
            .tab_mut()
            .is_some_and(|tab| tab.resize_if_settled(now));
        redraw_if(changed)
    }

    /// How long from `now` until [`Core::settle_size`] has work.
    pub fn settles_in(&self, now: Duration) -> Option<Duration> {
        self.output.tab()?.resize_in(now)
    }

    /// The wheel turned `notches`, up when positive, with the mouse at `at`.
    pub fn wheel(&mut self, notches: f32, at: Point) -> Vec<WindowCommand> {
        let Some(cell) = self.output.cell_at(at) else {
            return Vec::new();
        };
        let Some(tab) = self.output.tab_mut() else {
            return Vec::new();
        };
        let moved = tab.wheel(notches, cell).unwrap_or_else(|error| {
            log_warn!("the wheel did not reach the shell: {error}");
            false
        });
        redraw_if(moved)
    }

    /// Moves the view a screen back in the history, or forward.
    pub fn page(&mut self, back: bool) -> Vec<WindowCommand> {
        let moved = self.output.tab_mut().is_some_and(|tab| tab.page(back));
        redraw_if(moved)
    }

    /// What of the terminal is drawn. With the editor in use the output is
    /// anchored at the bottom (T-D22): what is over the prompt ends where the
    /// room does, and while a command runs, so does what it wrote.
    pub(super) fn layout(&self) -> Option<Layout> {
        let Place { full, seen, cell } = self.output.place?;
        let tab = self.output.tab()?;
        let terminal = tab.terminal();
        let back = isize::try_from(tab.back()).unwrap_or(isize::MAX);
        let anchored = self.typing.on
            && tab.integrated()
            && terminal.shell != ShellState::Unknown
            && !terminal.on_alternate_screen();
        if !anchored {
            return Some(Layout {
                area: full,
                first: -back,
                hidden: 0,
            });
        }
        let screen = terminal.screen();
        // The prompt's rows are not drawn while the editor shows it.
        let end = match self.typing.prompt(tab) {
            Some(prompt) => (screen.cursor().row + 1).saturating_sub(prompt.len()),
            None => screen.used_rows(),
        };
        let fit = usize::try_from(seen.height / cell.height.max(1)).unwrap_or(0);
        let history = screen.history_len();
        let shown = fit.min(history + end).min(screen.rows());
        let base = end as isize - shown as isize;
        let oldest = -isize::try_from(history).unwrap_or(isize::MAX);
        let tall = u32::try_from(shown).unwrap_or(0) * cell.height;
        let under = i32::try_from(seen.height.saturating_sub(tall)).unwrap_or(0);
        Some(Layout {
            area: Rect {
                y: seen.y.saturating_add(under),
                height: tall,
                ..seen
            },
            first: base.saturating_sub(back).max(oldest),
            hidden: usize::try_from(base).unwrap_or(0),
        })
    }

    /// The rows that changed since the last call, to draw them: it forgets
    /// them, so it is asked only right before drawing.
    pub fn take_damage(&mut self) -> Damage {
        let hidden = self.layout().map_or(0, |layout| layout.hidden);
        match self.output.tab_mut() {
            Some(tab) => {
                tab.hide(hidden);
                tab.terminal_mut().screen_mut().take_damage()
            }
            None => Damage::default(),
        }
    }

    /// What the renderer draws of the terminal, if there is one.
    pub fn picture<'a>(&'a self, damage: &'a Damage) -> Option<TerminalPicture<'a>> {
        let cell = self.output.place?.cell;
        let Layout { area, first, .. } = self.layout()?;
        Some(TerminalPicture {
            terminal: self.output.tab()?.terminal(),
            damage,
            first,
            area,
            palette: palette(self.theme),
            adjust_contrast: self.settings.adjust_contrast,
            cell,
            letters: pixels_of_points(self.settings.font.size, self.scale),
            // One logical pixel; a scale is never negative.
            line: self.scale.round().max(1.0) as u32,
            focused: self.focused,
        })
    }
}
