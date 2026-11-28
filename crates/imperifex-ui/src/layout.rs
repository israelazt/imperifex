//! Placing things. Every margin and padding is a step of the spacing scale: these
//! functions take a [`Space`], never a number.

use imperifex_domain::draw::Rect;
use imperifex_domain::space::Space;

/// `outer` with `space` taken off every side. What does not fit is left empty,
/// never negative.
pub fn inset(outer: Rect, space: Space, scale: f32) -> Rect {
    let padding = space.px(scale);
    let shift = |size: u32| signed(padding.min(size / 2));
    Rect {
        x: outer.x.saturating_add(shift(outer.width)),
        y: outer.y.saturating_add(shift(outer.height)),
        width: outer.width.saturating_sub(padding.saturating_mul(2)),
        height: outer.height.saturating_sub(padding.saturating_mul(2)),
    }
}

/// Items of the `widths` given, side by side from the left of `outer`, as tall
/// as it, with `gap` between them. What goes past its right side is cut.
pub fn row(outer: Rect, widths: &[u32], gap: Space, scale: f32) -> Vec<Rect> {
    let gap = gap.px(scale);
    let mut used = 0u32;
    widths
        .iter()
        .map(|&wanted| {
            let start = used.min(outer.width);
            let item = Rect {
                x: outer.x.saturating_add(signed(start)),
                width: wanted.min(outer.width - start),
                ..outer
            };
            used = used.saturating_add(wanted).saturating_add(gap);
            item
        })
        .collect()
}

fn signed(pixels: u32) -> i32 {
    i32::try_from(pixels).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn an_inset_takes_the_step_off_every_side_at_the_windows_scale() {
        let outer = rect(10, 20, 200, 100);
        assert_eq!(inset(outer, Space::L, 1.0), rect(26, 36, 168, 68));
        assert_eq!(inset(outer, Space::L, 1.5), rect(34, 44, 152, 52));
    }

    #[test]
    fn an_inset_larger_than_the_rectangle_leaves_it_empty_inside_it() {
        let found = inset(rect(5, 5, 10, 40), Space::L, 1.0);
        assert_eq!(found, rect(10, 21, 0, 8));
    }

    #[test]
    fn a_row_puts_items_side_by_side_with_the_gap_between() {
        let found = row(rect(4, 8, 100, 30), &[20, 30, 10], Space::S, 1.0);
        let expected = [rect(4, 8, 20, 30), rect(32, 8, 30, 30), rect(70, 8, 10, 30)];
        assert_eq!(found, expected);
    }

    #[test]
    fn what_does_not_fit_in_a_row_is_cut() {
        let found = row(rect(0, 0, 50, 10), &[40, 40, 40], Space::Xs, 1.0);
        assert_eq!(
            found,
            [rect(0, 0, 40, 10), rect(44, 0, 6, 10), rect(50, 0, 0, 10)]
        );
    }
}
