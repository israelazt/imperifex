//! The blocks of the window: where each one in view is, which one is chosen
//! and copying what its command wrote.

use imperifex_domain::blocks::{duration, output, seen, SeenBlock};
use imperifex_domain::view::{BlockView, CommandRow, RanView};

use super::output::Layout;
use super::Core;

impl Core {
    /// The blocks in view, from the top down; none without the integration or
    /// with a program on the whole screen.
    pub(super) fn block_views(&self) -> Vec<BlockView> {
        let Some(tab) = self.output.tab().filter(|tab| tab.integrated()) else {
            return Vec::new();
        };
        let terminal = tab.terminal();
        let (Some(Layout { area, first, .. }), Some(cell)) = (self.layout(), self.output.cell())
        else {
            return Vec::new();
        };
        if terminal.on_alternate_screen() {
            return Vec::new();
        }
        let tall = cell.height.max(1);
        let rows = usize::try_from(area.height / tall).unwrap_or(0);
        let pixels = |rows: usize| u32::try_from(rows).unwrap_or(0).saturating_mul(tall);
        let down = |rows: usize| {
            let down = i32::try_from(pixels(rows)).unwrap_or(i32::MAX);
            area.y.saturating_add(down)
        };
        let view = |block: SeenBlock| BlockView {
            id: block.id,
            top: down(block.from),
            height: pixels(block.to - block.from),
            starts_here: block.starts_here,
            command: block.command.map(|(row, columns)| {
                let written = u32::try_from(columns).unwrap_or(u32::MAX);
                CommandRow {
                    y: down(row),
                    height: tall,
                    free: area
                        .width
                        .saturating_sub(written.saturating_mul(cell.width)),
                }
            }),
            ran: block.ran.map(|ran| RanView {
                code: ran.code,
                took: ran.millis.map(|millis| duration(self.language, millis)),
            }),
            selected: self.chosen_block == Some(block.id),
        };
        seen(terminal, first, rows).into_iter().map(view).collect()
    }

    /// Chooses a block, or lets go of it if it was the one chosen.
    pub(super) fn choose_block(&mut self, id: u32) {
        self.chosen_block = (self.chosen_block != Some(id)).then_some(id);
    }

    /// Copies what the command of a block wrote.
    pub(super) fn copy_block(&self, id: u32) {
        let text = self.output.tab().and_then(|tab| output(tab.terminal(), id));
        if let Some(text) = text {
            // Windows programs expect both characters at each line's end.
            self.copy(&text.replace('\n', "\r\n"));
        }
    }
}
