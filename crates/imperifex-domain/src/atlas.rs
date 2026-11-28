//! Where each glyph drawn so far is in the texture that holds them. They are
//! packed in shelves: rows as tall as their first glyph, filled from the left.
//! Nothing is taken out one by one: when it is full it is emptied.

use std::collections::HashMap;

/// A glyph as it is drawn: which one, of which face, at what size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    /// The number the text system gave the face: a family in one weight and
    /// style, so bold and italic are faces of their own.
    pub face: u32,
    pub glyph: u16,
    /// The size of the letters in 64ths of a physical pixel.
    pub size: u32,
}

/// A glyph's place in the texture, and where its picture goes from the pen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Slot {
    pub x: u32,
    pub y: u32,
    /// 0 for a glyph with nothing to see, such as a space.
    pub width: u32,
    pub height: u32,
    /// From the pen to its left side, and from the baseline to its top.
    pub left: i32,
    pub top: i32,
}

#[derive(Debug)]
struct Shelf {
    top: u32,
    height: u32,
    used: u32,
}

#[derive(Debug)]
pub struct Atlas {
    /// The texture is square.
    side: u32,
    shelves: Vec<Shelf>,
    slots: HashMap<GlyphKey, Slot>,
}

impl Atlas {
    pub fn new(side: u32) -> Self {
        Self {
            side,
            shelves: Vec::new(),
            slots: HashMap::new(),
        }
    }

    pub fn get(&self, key: GlyphKey) -> Option<Slot> {
        self.slots.get(&key).copied()
    }

    /// Gives `key` a place for a picture of the size in `picture`, whose `x`
    /// and `y` are ignored. `None` when there is no room left: after
    /// [`Atlas::clear`] it fits, unless it is larger than the whole texture.
    pub fn insert(&mut self, key: GlyphKey, picture: Slot) -> Option<Slot> {
        let slot = if picture.width == 0 || picture.height == 0 {
            Slot {
                x: 0,
                y: 0,
                ..picture
            }
        } else {
            let (x, y) = self.place(picture.width, picture.height)?;
            Slot { x, y, ..picture }
        };
        self.slots.insert(key, slot);
        Some(slot)
    }

    pub fn clear(&mut self) {
        self.shelves.clear();
        self.slots.clear();
    }

    /// On the lowest shelf it fits in, or on a new one under the last.
    fn place(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let side = self.side;
        if width > side || height > side {
            return None;
        }
        let fitting = self
            .shelves
            .iter_mut()
            .filter(|shelf| shelf.height >= height && side - shelf.used >= width)
            .min_by_key(|shelf| shelf.height);
        if let Some(shelf) = fitting {
            let x = shelf.used;
            shelf.used += width;
            return Some((x, shelf.top));
        }
        let top = self.shelves.last().map_or(0, |s| s.top + s.height);
        if side - top < height {
            return None;
        }
        self.shelves.push(Shelf {
            top,
            height,
            used: width,
        });
        Some((0, top))
    }
}

#[cfg(test)]
mod tests;
