use udled::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

impl Position {
    pub fn new(line: usize, character: usize) -> Position {
        Position { line, character }
    }

    pub fn into_span(self, source: &str) -> Option<Span> {
        let mut line = 0;

        if self.line == 0 {
            return Some(Span::new(
                self.character as usize,
                (self.character as usize) + 1,
            ));
        }

        for (idx, v) in source.bytes().enumerate() {
            if (v as char) == '\n' {
                line += 1;
            }

            if line == self.line {
                let idx = idx + self.character as usize;
                return Some(Span::new(idx, idx + 1));
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Range {
        Range { start, end }
    }

    pub fn from_span(source: &str, span: Span) -> Option<Range> {
        let mut s_line = 0;
        let mut s_col = 0;
        for (k, v) in source.bytes().enumerate() {
            if k == span.start {
                break;
            }
            if (v as char) == '\n' {
                s_line += 1;
                s_col = 0;
            } else {
                s_col += 1;
            }
        }

        let mut e_line = s_line;
        let mut e_col = s_col;

        for (k, v) in source.bytes().enumerate().skip(span.start) {
            if k == span.end {
                return Some(Range::new(
                    Position::new(s_line, s_col),
                    Position::new(e_line, e_col),
                ));
            }
            if (v as char) == '\n' {
                e_line += 1;
                e_col = 0;
            } else {
                e_col += 1;
            }
        }

        None
    }

    pub fn into_span(self, source: &str) -> Option<Span> {
        let start = self.start.into_span(source)?;
        let end = self.end.into_span(source)?;
        Some(start + end)
    }
}
