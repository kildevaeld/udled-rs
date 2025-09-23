mod sealed {

    pub trait Sealed {}

    impl<'a> Sealed for &'a str {}

    impl Sealed for char {}
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
        *self == '\n'
    }
}

pub trait LineBreak: sealed::Sealed {
    fn count_linebreak(&self) -> usize;
}

impl<'a> LineBreak for &'a str {
    fn count_linebreak(&self) -> usize {
        self.chars()
            .fold(0, |p, c| p + if c.is_linebreak() { 1 } else { 0 })
    }
}

impl LineBreak for char {
    fn count_linebreak(&self) -> usize {
        if self.is_linebreak() {
            1
        } else {
            0
        }
    }
}
