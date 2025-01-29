/// Generates an enum with a method to get the enum variant by its value.
/// 
/// The generated enum derives `PartialEq`.
/// 
/// Example:
/// 
/// ```rust
/// value_enum! {
///     MyEnum: i32 {
///         A = 1,
///         B = 2,
///         C = 3,
///     }
/// }
/// 
/// let a = MyEnum::by_value(1);
/// ```
macro_rules! value_enum {
    ($name:ident: $t:ty { $($(#[$_:meta])* $entry:ident = $value:expr),* $(,)? }) => {
        #[derive(PartialEq)]
        pub enum $name {     
            $($entry = $value,)*
        }

        impl $name {
            /// Returns the enum variant corresponding to the given value.
            pub fn by_value(value: $t) -> Option<Self> {
                match value {
                    $($value => Some(Self::$entry),)*
                    _ => None
                }
            }
        }
    };
}

pub(crate) use value_enum;