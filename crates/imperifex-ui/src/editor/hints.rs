//! What the keys of the editor do, under its box; or why a command was not
//! sent.

use imperifex_domain::draw::{DrawList, UiFont};
use imperifex_domain::i18n::{text, Text};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::space::Space;
use imperifex_domain::view::{EditorView, Model};

use super::signed;
use crate::metrics;
use crate::text::Ink;

/// Each key and what it does.
const KEYS: [(Text, Text); 4] = [
    (Text::KeyEnter, Text::EditorRun),
    (Text::KeyShiftEnter, Text::EditorNewLine),
    (Text::KeyRight, Text::EditorAccept),
    (Text::KeyUp, Text::EditorHistory),
];

/// The line from `at`, to the right.
pub fn draw(
    model: &Model,
    view: &EditorView,
    (left, y): (i32, i32),
    measure: &dyn TextMeasure,
    into: &mut DrawList,
) {
    let ink = |color| Ink {
        font: UiFont::Text,
        size: metrics::px(metrics::SECONDARY_TEXT, model.scale),
        color,
    };
    if view.refused {
        let why = text(model.language, Text::EditorRefused);
        into.texts
            .push(ink(model.palette.warning).run((left, y), why.into()));
        return;
    }
    let (tight, apart) = (Space::Xs.px(model.scale), Space::L.px(model.scale));
    let mut x = left;
    let mut say = |said: Text, color, after: u32| {
        let words = text(model.language, said);
        let letters = ink(color);
        into.texts.push(letters.run((x, y), words.into()));
        let width = measure.width(words, letters.font, letters.size);
        x = x.saturating_add(signed(width + after));
    };
    for (key, does) in KEYS {
        say(key, model.palette.text, tight);
        say(does, model.palette.muted, apart);
    }
}
