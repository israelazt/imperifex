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
        params: Params<'a>,
        /// Private markers and intermediate bytes, as the `?` of `ESC[?25l`.
        intermediates: &'a [u8],
        action: char,
    },
    /// `ESC` and a final byte that is not `[` nor `]`.
    Esc { intermediates: &'a [u8], action: u8 },
    /// `ESC ]`: the parts between `;`.
    Osc(&'a [&'a [u8]]),
}

/// The numbers of a CSI sequence. A parameter is separated from the next by
/// `;`, and may have parts of its own separated by `:`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Params<'a> {
    /// Every number in order; 0 where it was left out.
    values: &'a [u16],
    /// Bit `n` is set when value `n` came after a `:`: it belongs to the same
    /// parameter as the one before it. Only the first 32 values can be marked.
    joined: u32,
}

impl<'a> Params<'a> {
    pub fn new(values: &'a [u16], joined: u32) -> Self {
        Self { values, joined }
    }

    fn is_joined(&self, index: usize) -> bool {
        index < 32 && self.joined & (1 << index) != 0
    }

    /// Each parameter with its parts; none of them is empty.
    pub fn groups(&self) -> impl Iterator<Item = &'a [u16]> + '_ {
        let mut from = 0;
        std::iter::from_fn(move || {
            if from >= self.values.len() {
                return None;
            }
            let mut to = from + 1;
            while to < self.values.len() && self.is_joined(to) {
                to += 1;
            }
            let group = &self.values[from..to];
            from = to;
            Some(group)
        })
    }

    /// The first number of a parameter; 0 if there is no such parameter.
    pub fn get(&self, index: usize) -> u16 {
        self.groups().nth(index).map_or(0, |group| group[0])
    }
}

/// Parameters without parts.
impl<'a> From<&'a [u16]> for Params<'a> {
    fn from(values: &'a [u16]) -> Self {
        Self::new(values, 0)
    }
}
