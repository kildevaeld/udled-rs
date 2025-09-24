#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn from(input: &str, byte_position: usize) -> Option<Location> {
        byte_to_line(input, byte_position)
    }
}

fn byte_to_line(input: &str, pos: usize) -> Option<Location> {
    let mut line = 0;
    if !input.is_char_boundary(pos) || pos > input.len() {
        return None;
    }

    let mut row = 0;
    let mut chars = input.char_indices().peekable();

    loop {
        let Some((idx, char)) = chars.next() else {
            break;
        };

        if idx == pos {
            break;
        }

        let mut is_ws = char == '\n';
        if !is_ws && char == '\r' {
            if chars.peek().map(|m| m.1) == Some('\n') {
                let _ = chars.next();
                is_ws = true;
            }
        }

        if is_ws {
            line += 1;
            row = 0;
        } else {
            row += 1;
        }
    }

    Some(Location { line, column: row })
}

pub(crate) const fn is_utf8_char_boundary(this: u8) -> bool {
    // This is bit magic equivalent to: b < 128 || b >= 192
    (this as i8) >= -0x40
}
