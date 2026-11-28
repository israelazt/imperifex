use super::*;

fn text(text: &str) -> KeyInput {
    KeyInput {
        text: Some(text.into()),
        ..KeyInput::default()
    }
}

fn key(key: NamedKey) -> KeyInput {
    KeyInput {
        key: Some(key),
        ..KeyInput::default()
    }
}

fn sent(input: &KeyInput) -> String {
    String::from_utf8(bytes(input, Modes::default())).expect("text")
}

#[test]
fn text_goes_as_it_is_in_utf_8_and_ctrl_with_a_letter_is_its_control_character() {
    for typed in ["a", "ñ€", " ", "\u{3}", "\u{12}", "\u{c}"] {
        assert_eq!(sent(&text(typed)), typed, "{typed:?}");
    }
    assert_eq!(sent(&KeyInput::default()), "");
    let ctrl_c = KeyInput {
        ctrl: true,
        ..text("\u{3}")
    };
    assert_eq!(sent(&ctrl_c), "\u{3}");
}

#[test]
fn alt_puts_an_escape_before_the_letter() {
    let alt_v = KeyInput {
        alt: true,
        ..text("v")
    };
    assert_eq!(sent(&alt_v), "\x1bv");
}

#[test]
fn what_alt_gr_writes_is_only_text_but_ctrl_and_alt_without_text_stay() {
    let held = |typed: &str| KeyInput {
        ctrl: true,
        alt: true,
        ..text(typed)
    };
    let at = held("@").without_alt_graph();
    assert_eq!((at.ctrl, at.alt, sent(&at)), (false, false, "@".into()));
    let control = held("\u{3}").without_alt_graph();
    assert!(control.ctrl && control.alt);
    let arrow = KeyInput {
        ctrl: true,
        alt: true,
        ..key(NamedKey::Left)
    };
    assert_eq!(sent(&arrow.without_alt_graph()), "\x1b[1;7D");
}

#[test]
fn each_named_key_sends_its_bytes_whatever_text_comes_with_it() {
    for (named, expected) in [
        (NamedKey::Enter, "\r"),
        (NamedKey::Backspace, "\x7f"),
        (NamedKey::Tab, "\t"),
        (NamedKey::Escape, "\x1b"),
        (NamedKey::Up, "\x1b[A"),
        (NamedKey::Down, "\x1b[B"),
        (NamedKey::Right, "\x1b[C"),
        (NamedKey::Left, "\x1b[D"),
        (NamedKey::Home, "\x1b[H"),
        (NamedKey::End, "\x1b[F"),
        (NamedKey::Insert, "\x1b[2~"),
        (NamedKey::Delete, "\x1b[3~"),
        (NamedKey::PageUp, "\x1b[5~"),
        (NamedKey::PageDown, "\x1b[6~"),
    ] {
        let with_text = KeyInput {
            text: Some("\u{8}".into()),
            ..key(named)
        };
        for input in [key(named), with_text] {
            assert_eq!(sent(&input), expected, "{named:?}");
        }
    }
}

#[test]
fn the_function_keys_follow_xterms_numbers() {
    let found: Vec<String> = (1..=12)
        .map(|number| sent(&key(NamedKey::Function(number))))
        .collect();
    let expected = [
        "\x1bOP", "\x1bOQ", "\x1bOR", "\x1bOS", "\x1b[15~", "\x1b[17~", "\x1b[18~", "\x1b[19~",
        "\x1b[20~", "\x1b[21~", "\x1b[23~", "\x1b[24~",
    ];
    assert_eq!(found, expected);
    assert_eq!(sent(&key(NamedKey::Function(13))), "");
}

#[test]
fn modifiers_go_inside_the_sequence_and_shift_tab_goes_back() {
    let with = |named, shift, ctrl| KeyInput {
        shift,
        ctrl,
        ..key(named)
    };
    assert_eq!(sent(&with(NamedKey::Left, false, true)), "\x1b[1;5D");
    assert_eq!(sent(&with(NamedKey::End, true, false)), "\x1b[1;2F");
    assert_eq!(sent(&with(NamedKey::Delete, false, true)), "\x1b[3;5~");
    assert_eq!(sent(&with(NamedKey::Function(1), true, false)), "\x1b[1;2P");
    assert_eq!(sent(&with(NamedKey::Tab, true, false)), "\x1b[Z");
    assert_eq!(sent(&with(NamedKey::Backspace, false, true)), "\u{8}");
}

#[test]
fn shift_or_alt_enter_is_a_new_line_only_where_the_program_takes_pastes() {
    let pastes = Modes {
        bracketed_paste: true,
        ..Modes::default()
    };
    let shift = KeyInput {
        shift: true,
        ..key(NamedKey::Enter)
    };
    assert_eq!(bytes(&shift, pastes), b"\x1b\r");
    assert_eq!(bytes(&shift, Modes::default()), b"\r");
    let alt = KeyInput {
        alt: true,
        ..key(NamedKey::Enter)
    };
    assert_eq!(bytes(&alt, Modes::default()), b"\x1b\r");
    let ctrl = KeyInput {
        ctrl: true,
        ..key(NamedKey::Enter)
    };
    assert_eq!(bytes(&ctrl, pastes), b"\n");
    assert_eq!(bytes(&key(NamedKey::Enter), pastes), b"\r");
}

#[test]
fn the_arrows_change_when_the_program_asks_for_application_keys() {
    let modes = Modes {
        application_cursor_keys: true,
        ..Modes::default()
    };
    assert_eq!(bytes(&key(NamedKey::Up), modes), b"\x1bOA");
    assert_eq!(bytes(&key(NamedKey::Left), modes), b"\x1bOD");
    assert_eq!(bytes(&key(NamedKey::Enter), modes), b"\r");
}
