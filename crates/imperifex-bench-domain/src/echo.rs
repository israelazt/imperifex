//! What the bench's echo program answers to what is typed into it. It is the
//! same in every terminal, so that timing a key through it compares the
//! terminals and not their shells (D14).

/// What the program writes when it starts.
pub const ECHO_PROMPT: &str = "IMPERIFEX-ECHO> ";

/// Takes back the last character: back, blank it, back.
const ERASE: &[u8] = b"\x08 \x08";
const DELETE: u8 = 0x7F;
const BACKSPACE: u8 = 0x08;
const QUIT: u8 = b'q';

/// What to write for some input, and whether it asked to leave.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Reply {
    pub bytes: Vec<u8>,
    pub quit: bool,
}

/// Printable ASCII is written back as it came and a backspace takes one back;
/// `q` leaves, and what follows it is not looked at. Anything else is ignored.
pub fn echo_reply(input: &[u8]) -> Reply {
    let mut reply = Reply::default();
    for &byte in input {
        match byte {
            QUIT => {
                reply.quit = true;
                break;
            }
            DELETE | BACKSPACE => reply.bytes.extend_from_slice(ERASE),
            b' '..=b'~' => reply.bytes.push(byte),
            _ => {}
        }
    }
    reply
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrote(input: &[u8]) -> Vec<u8> {
        echo_reply(input).bytes
    }

    #[test]
    fn printable_characters_come_back_as_they_were() {
        assert_eq!(wrote(b"ab ~!"), b"ab ~!");
    }

    #[test]
    fn either_backspace_takes_a_character_back() {
        assert_eq!(wrote(&[DELETE]), ERASE);
        assert_eq!(wrote(&[BACKSPACE]), ERASE);
    }

    #[test]
    fn control_bytes_and_what_is_not_ascii_are_ignored() {
        assert_eq!(wrote(b"\r\n\x1b\x03\xc3\xb1"), b"");
    }

    #[test]
    fn q_leaves_and_nothing_after_it_is_looked_at() {
        let reply = echo_reply(b"abqcd");
        assert_eq!((reply.bytes.as_slice(), reply.quit), (&b"ab"[..], true));
        assert!(!echo_reply(b"abc").quit);
    }
}
