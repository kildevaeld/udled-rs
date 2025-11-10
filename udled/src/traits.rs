use crate::Span;

mod sealed {

    pub trait Sealed {}

    impl<'a> Sealed for &'a str {}

    impl Sealed for char {}

    impl<'a> Sealed for &'a [u8] {}
}

pub trait StringExt: sealed::Sealed {
    fn is_ascii_alphanumeric(&self) -> bool;
    fn is_ascii_alphabetic(&self) -> bool;
    fn is_ascii_whitespace(&self) -> bool;
    fn is_ascii_punctuation(&self) -> bool;
    fn is_whitespace(&self) -> bool;
    fn is_linebreak(&self) -> bool;

    fn is_ascii_digit(&self) -> bool;
    fn is_digit(&self, radix: u32) -> bool;

    fn is_alphanumeric(&self) -> bool;
    fn is_alphabetic(&self) -> bool;
}

impl<'a> StringExt for &'a str {
    fn is_ascii_alphanumeric(&self) -> bool {
        self.chars().all(|m| m.is_ascii_alphanumeric())
    }

    fn is_ascii_punctuation(&self) -> bool {
        self.chars().all(|m| m.is_ascii_punctuation())
    }

    fn is_ascii_alphabetic(&self) -> bool {
        self.chars().all(|m| m.is_ascii_alphabetic())
    }

    fn is_ascii_whitespace(&self) -> bool {
        self.chars().all(|m| m.is_ascii_whitespace())
    }

    fn is_whitespace(&self) -> bool {
        self.chars().all(|m| m.is_whitespace())
    }

    fn is_ascii_digit(&self) -> bool {
        self.chars().all(|m| m.is_ascii_digit())
    }

    fn is_digit(&self, radix: u32) -> bool {
        self.chars().all(|m| m.is_digit(radix))
    }

    fn is_linebreak(&self) -> bool {
        self.chars().all(|m| m.is_linebreak())
    }

    fn is_alphanumeric(&self) -> bool {
        self.chars().all(|m| m.is_alphanumeric())
    }
    fn is_alphabetic(&self) -> bool {
        self.chars().all(|m| m.is_alphabetic())
    }
}

impl StringExt for char {
    fn is_ascii_alphanumeric(&self) -> bool {
        (*self).is_ascii_alphanumeric()
    }

    fn is_digit(&self, radix: u32) -> bool {
        (*self).is_digit(radix)
    }

    fn is_ascii_alphabetic(&self) -> bool {
        (*self).is_ascii_alphabetic()
    }

    fn is_ascii_whitespace(&self) -> bool {
        (*self).is_ascii_whitespace()
    }

    fn is_ascii_punctuation(&self) -> bool {
        (*self).is_ascii_punctuation()
    }

    fn is_whitespace(&self) -> bool {
        (*self).is_whitespace()
    }

    fn is_ascii_digit(&self) -> bool {
        (*self).is_ascii_digit()
    }

    fn is_alphanumeric(&self) -> bool {
        (*self).is_alphanumeric()
    }
    fn is_alphabetic(&self) -> bool {
        (*self).is_alphabetic()
    }

    fn is_linebreak(&self) -> bool {
        *self == '\n' || *self == '\r' || *self == '\u{2028}' || *self == '\u{2029}'
    }
}

pub trait LineBreaks: sealed::Sealed {
    fn count_linebreak(&self) -> usize;
}

impl<'a> LineBreaks for &'a str {
    fn count_linebreak(&self) -> usize {
        self.chars()
            .fold(0, |p, c| p + if c.is_linebreak() { 1 } else { 0 })
    }
}

impl<'a> LineBreaks for &'a [u8] {
    fn count_linebreak(&self) -> usize {
        self.iter().fold(0, |p, c| {
            p + if (*c as char).is_linebreak() { 1 } else { 0 }
        })
    }
}

impl LineBreaks for char {
    fn count_linebreak(&self) -> usize {
        if self.is_linebreak() {
            1
        } else {
            0
        }
    }
}

pub trait AsChar {
    fn as_char(&self) -> Option<char>;
}

impl AsChar for char {
    fn as_char(&self) -> Option<char> {
        Some(*self)
    }
}

impl AsChar for u8 {
    fn as_char(&self) -> Option<char> {
        Some(*self as _)
    }
}

impl AsChar for u32 {
    fn as_char(&self) -> Option<char> {
        char::from_u32(*self)
    }
}

pub trait AsBytes<'a> {
    fn as_bytes(&self) -> &'a [u8];
}

impl<'a> AsBytes<'a> for &'a str {
    fn as_bytes(&self) -> &'a [u8] {
        (*self).as_bytes()
    }
}

impl<'a> AsBytes<'a> for &'a [u8] {
    fn as_bytes(&self) -> &'a [u8] {
        self
    }
}

pub trait AsStr<'a>: AsBytes<'a> {
    fn as_str(&self) -> &'a str;
}

impl<'a> AsStr<'a> for &'a str {
    fn as_str(&self) -> &'a str {
        self
    }
}

pub trait AsSlice<'a> {
    type Slice;
    fn sliced(&self, span: Span) -> Option<Self::Slice>;
}

impl<'a> AsSlice<'a> for &'a str {
    type Slice = &'a str;
    fn sliced(&self, span: Span) -> Option<Self::Slice> {
        span.slice(self)
    }
}

impl<'a> AsSlice<'a> for &'a [u8] {
    type Slice = &'a [u8];
    fn sliced(&self, span: Span) -> Option<Self::Slice> {
        if span.end > self.len() {
            return None;
        }

        Some(&self[span.start..span.end])
    }
}
