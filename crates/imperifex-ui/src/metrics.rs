//! The sizes of the interface, in logical pixels (see «Diseño: medidas» in the
//! plan). These are not steps of the spacing scale: margins and paddings are,
//! and take a [`imperifex_domain::space::Space`].

pub const TITLE_BAR_HEIGHT: f32 = 40.0;
pub const BUTTON_WIDTH: f32 = 46.0;
/// How far in from the window's sides the mouse changes its size.
pub const RESIZE_EDGE: f32 = 8.0;
pub const TAB_HEIGHT: f32 = 32.0;
pub const TAB_RADIUS: f32 = 6.0;
pub const STATUS_BAR_HEIGHT: f32 = 28.0;
pub const BLOCK_RADIUS: f32 = 4.0;
pub const RESULT_ICON: f32 = 14.0;
pub const EDITOR_RADIUS: f32 = 8.0;
pub const CURSOR_WIDTH: f32 = 2.0;
/// The line around a box.
pub const BORDER: f32 = 1.0;

/// The sizes of letters and icons.
pub const TEXT: f32 = 13.0;
pub const SECONDARY_TEXT: f32 = 12.0;
pub const BUTTON_ICON: f32 = 10.0;
pub const TAB_ICON: f32 = 16.0;
pub const STATUS_ICON: f32 = 12.0;

/// A logical measure in whole physical pixels at the window's `scale`.
pub fn px(logical: f32, scale: f32) -> u32 {
    // Rounded and never negative before the cast, which then cannot wrap.
    (logical * scale).round().max(0.0) as u32
}
