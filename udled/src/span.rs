use core::fmt;

pub trait WithSpan {
    fn span(&self) -> Span;
}

impl<L, R> WithSpan for crate::Either<L, R>
where
    L: WithSpan,
    R: WithSpan,
{
    fn span(&self) -> Span {
        match self {
            crate::Either::Left(m) => m.span(),
            crate::Either::Right(m) => m.span(),
        }
    }
}

impl<T> WithSpan for Option<T>
where
    T: WithSpan,
{
    fn span(&self) -> Span {
        match self {
            Some(s) => s.span(),
            None => Span::new(0, 0),
        }
    }
}

impl<T> WithSpan for alloc::vec::Vec<T>
where
    T: WithSpan,
{
    fn span(&self) -> Span {
        if self.is_empty() {
            Span::default()
        } else {
            let (first, last) = match (self.first(), self.last()) {
                (Some(first), Some(last)) => (first.span(), last.span()),
                _ => return Span::default(),
            };

            first + last
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    pub const fn is_valid(&self) -> bool {
        self.start < self.end
    }

    pub fn slice<'a>(&self, input: &'a str) -> Option<&'a str> {
        if !self.is_valid() || self.end > input.len() {
            None
        } else {
            Some(&input[self.start..self.end])
        }
    }

    pub const fn len(&self) -> usize {
        if self.is_valid() {
            self.end - self.start
        } else {
            0
        }
    }

    pub const fn range(&self) -> core::ops::Range<usize> {
        self.start..self.end
    }

    pub const fn contains(&self, other: Span) -> bool {
        other.start >= self.start && other.end <= self.end
    }

    pub const fn with_end(self, end: usize) -> Span {
        Span {
            start: self.start,
            end,
        }
    }

    pub const fn with_start(self, start: usize) -> Span {
        Span {
            start,
            end: self.end,
        }
    }
}

impl core::ops::RangeBounds<usize> for Span {
    fn start_bound(&self) -> core::ops::Bound<&usize> {
        core::ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> core::ops::Bound<&usize> {
        core::ops::Bound::Excluded(&self.end)
    }
}

impl WithSpan for Span {
    fn span(&self) -> Span {
        *self
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}]", self.start, self.end)
    }
}

impl core::ops::Add for Span {
    type Output = Span;
    fn add(mut self, rhs: Self) -> Self::Output {
        if !rhs.is_valid() {
            return self;
        } else if !self.is_valid() {
            return rhs;
        }

        if rhs.start < self.start {
            self.start = rhs.start;
        }
        if rhs.end > self.end {
            self.end = rhs.end;
        }

        self
    }
}

impl core::ops::AddAssign for Span {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

macro_rules! withspan_impl {
    ($first: ident) => {

        impl<$first: WithSpan> WithSpan for ($first, ) {
            fn span(&self) -> Span {
                self.0.span()
            }
        }


    };
    ($first: ident $($rest:ident)*) => {
        withspan_impl!($($rest)*);

        #[allow(non_snake_case)]
        impl<$first: WithSpan, $($rest: WithSpan),*> WithSpan for ($first, $($rest),*) {
            fn span(&self) -> Span {
                let ($first, $($rest),*) = self;
                $first.span()  $(+ $rest.span())*
            }
        }
    };
}

withspan_impl!(T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16);

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! span {
        ($start:expr, $end: expr) => {
            Span::new($start, $end)
        };
    }

    #[test]
    fn test_invalid() {
        assert!(!Span::default().is_valid());
        assert!(!Span::new(0, 0).is_valid());
        assert!(!Span::new(2, 1).is_valid());
    }

    #[test]
    fn len() {
        assert_eq!(Span::default().len(), 0);
        assert_eq!(Span::new(2, 1).len(), 0);
        assert_eq!(Span::new(0, 1).len(), 1);
        assert_eq!(Span::new(2, 4).len(), 2);
    }

    #[test]
    fn test_add() {
        assert_eq!(span!(0, 36), span!(0, 30) + span!(20, 36));
        assert_eq!(span!(0, 40), span!(0, 10) + span!(30, 40));
        assert_eq!(span!(10, 30), span!(15, 17) + span!(10, 30));
        assert_eq!(span!(0, 1), span!(0, 1) + span!(0, 0));
        assert_eq!(span!(0, 1), span!(0, 0) + span!(0, 1));
    }
}
