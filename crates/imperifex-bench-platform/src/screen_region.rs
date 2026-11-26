//! Where a region of the desktop falls on a monitor. Plain code, tested
//! everywhere.

use imperifex_bench_domain::desktop::Rect;

/// The corner of `region` counted from that of `monitor`, if the whole region
/// is on it; `None` if any part is not, or if the region has no area.
pub(crate) fn inside(monitor: Rect, region: Rect) -> Option<(u32, u32)> {
    let fits = region.w > 0
        && region.h > 0
        && region.x >= monitor.x
        && region.y >= monitor.y
        && region.x.checked_add(region.w)? <= monitor.x.checked_add(monitor.w)?
        && region.y.checked_add(region.h)? <= monitor.y.checked_add(monitor.h)?;
    if !fits {
        return None;
    }
    let left = u32::try_from(region.x - monitor.x).ok()?;
    let top = u32::try_from(region.y - monitor.y).ok()?;
    Some((left, top))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    const SECOND: Rect = Rect {
        x: 1920,
        y: 0,
        w: 1920,
        h: 1080,
    };

    #[test]
    fn a_region_on_a_monitor_is_counted_from_its_corner() {
        assert_eq!(inside(SECOND, rect(2000, 40, 100, 100)), Some((80, 40)));
        assert_eq!(inside(SECOND, SECOND), Some((0, 0)));
    }

    #[test]
    fn a_region_that_sticks_out_on_any_side_is_not_on_it() {
        for region in [
            rect(1919, 0, 100, 100),
            rect(2000, -1, 100, 100),
            rect(3800, 0, 100, 100),
            rect(2000, 1000, 100, 100),
            rect(0, 0, 100, 100),
        ] {
            assert_eq!(inside(SECOND, region), None, "{region:?}");
        }
    }

    #[test]
    fn a_region_with_no_area_is_nowhere() {
        assert_eq!(inside(SECOND, rect(2000, 40, 0, 100)), None);
        assert_eq!(inside(SECOND, rect(2000, 40, 100, -1)), None);
    }
}
