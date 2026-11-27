//! What a shell writes, cut into pieces with a shape. What each piece means is
//! decided in [`crate::interpret`].

/// One piece of what a shell writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sequence<'a> {
    /// A character to show.
    Print(char),
    /// A control character, such as `\r`.
    Control(u8),
    /// `ESC [`, then the parameters and a final letter.
    Csi {
        /// One value per parameter; 0 when it was left out. Sub-parameters
        /// (after `:`) are not kept.
        params: &'a [u16],
        /// Private markers and intermediate bytes, as the `?` of `ESC[?25l`.
        intermediates: &'a [u8],
        action: char,
    },
    /// `ESC` and a final byte that is not `[` nor `]`.
    Esc { intermediates: &'a [u8], action: u8 },
    /// `ESC ]`: the parts between `;`.
    Osc(&'a [&'a [u8]]),
}
