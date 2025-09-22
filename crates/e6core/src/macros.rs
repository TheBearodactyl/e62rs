#[macro_export]
macro_rules! impl_display {
    ($type:ty, $name:expr, $color:ident, $($field:ident: $format:expr),*) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                writeln!(f, "{} {{", $name.$color())?;
                $(
                    writeln!(f, "  {}: {}", stringify!($field).yellow(), $format(&self.$field))?;
                )*
                writeln!(f, "}}")
            }
        }
    };
}

#[macro_export]
macro_rules! fmt_value {
    () => {
        |v| format!("{}", v)
    };
    (debug) => {
        |v| format!("{:?}", v)
    };
}
