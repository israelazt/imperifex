//! What the graphics card draws: one instance per rectangle, which the vertex
//! shader opens into its four corners. Portable, so that it is tested
//! everywhere.

use imperifex_domain::draw::{DrawList, Rect};
use imperifex_domain::theme::Rgb;

/// Laid out as `shaders.hlsl` reads it.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Instance {
    /// Left, top, width and height in physical pixels.
    pub area: [f32; 4],
    /// Red, green, blue and alpha, from 0 to 1.
    pub color: [f32; 4],
    /// For a glyph, where it is in the atlas: left, top, width and height.
    pub atlas: [f32; 4],
    /// 1 for a glyph, whose shape comes from the atlas; 0 for a rectangle.
    pub glyph: f32,
}

/// Puts the rectangles of `list` in `into`, in order. Those with nothing to see
/// are left out. Its texts wait until there are fonts.
pub fn fill(list: &DrawList, into: &mut Vec<Instance>) {
    into.clear();
    let visible = list
        .rects
        .iter()
        .filter(|(area, _)| area.width > 0 && area.height > 0);
    into.extend(visible.map(|&(area, color)| solid(area, color)));
}

fn solid(area: Rect, color: Rgb) -> Instance {
    let channel = |value: u8| f32::from(value) / 255.0;
    // A place on a screen is far inside what an `f32` holds exactly.
    Instance {
        area: [
            area.x as f32,
            area.y as f32,
            area.width as f32,
            area.height as f32,
        ],
        color: [channel(color.0), channel(color.1), channel(color.2), 1.0],
        ..Instance::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_rectangle_with_something_to_see_is_one_instance_in_order() {
        let area = |x, width| Rect {
            x,
            y: 2,
            width,
            height: 30,
        };
        let list = DrawList {
            rects: vec![
                (area(1, 20), Rgb(255, 0, 51)),
                (area(5, 0), Rgb::WHITE),
                (area(9, 4), Rgb::BLACK),
            ],
            ..DrawList::default()
        };
        let mut found = vec![Instance::default(); 7];
        fill(&list, &mut found);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].area, [1.0, 2.0, 20.0, 30.0]);
        assert_eq!(found[0].color, [1.0, 0.0, 0.2, 1.0]);
        assert_eq!((found[0].glyph, found[0].atlas), (0.0, [0.0; 4]));
        assert_eq!(found[1].area[0], 9.0);
    }
}
