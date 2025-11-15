#[macro_export]
macro_rules! any {
    [$one: expr] => {
        $one
    };
    [$first: expr, $($rest: expr),*] => {
        $crate::tokenizers::Or::new($first, $crate::any!($($rest),*))
    };

}
