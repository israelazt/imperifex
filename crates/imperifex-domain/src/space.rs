//! The spacing scale (T-D8): every margin and padding of the interface is one of
//! these steps. The idea of CSS's `rem`, and only the idea.

/// The base every step comes from, in logical pixels.
const BASE: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    /// Between an icon and its text.
    Xs,
    /// Between related things.
    S,
    /// The padding of buttons and tabs.
    M,
    /// The padding of panels, cards and the output.
    L,
    /// Between groups.
    Xl,
    /// Between sections.
    Xxl,
}

impl Space {
    /// How many times the base.
    pub fn rem(self) -> f32 {
        match self {
            Self::Xs => 0.25,
            Self::S => 0.5,
            Self::M => 0.75,
            Self::L => 1.0,
            Self::Xl => 1.5,
            Self::Xxl => 2.0,
        }
    }

    /// In physical pixels at the system's `scale` (1.0 is 100 %), whole so that
    /// nothing looks blurred.
    pub fn px(self, scale: f32) -> u32 {
        // Rounded, finite and never negative before the cast, which then
        // cannot wrap: a nonsense scale gives 0.
        (self.rem() * BASE * scale).round().max(0.0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEPS: [Space; 6] = [
        Space::Xs,
        Space::S,
        Space::M,
        Space::L,
        Space::Xl,
        Space::Xxl,
    ];

    #[test]
    fn the_scale_grows_with_the_windows_scale_in_whole_pixels() {
        let at = |scale| STEPS.map(|step| step.px(scale));
        assert_eq!(at(1.0), [4, 8, 12, 16, 24, 32]);
        assert_eq!(at(1.25), [5, 10, 15, 20, 30, 40]);
        assert_eq!(at(1.5), [6, 12, 18, 24, 36, 48]);
        assert_eq!(at(-1.0), [0; 6]);
    }
}
