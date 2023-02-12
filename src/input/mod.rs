mod btn;
mod key;

use std::{fmt::Display, str::FromStr};

pub use self::{btn::Btn, key::Key};

#[macro_export(local_inner_macros)]
macro_rules! __input_enum_internal {
    ($(#[$attr:meta])* ($($vis:tt)*) enum $N:ident { $($t:tt)* } impl TryFrom<$from:ty>;) => {
        __input_enum_internal!(@ENUM, $(#[$attr])*, ($($vis)*), $N, $($t)*);
        __input_enum_internal!(@IMPLS, $N, $from, $($t)*);
    };
    (@ENUM, $(#[$attr:meta])*, ($($vis:tt)*), $N:ident, $($tag:ident => $key:literal $evdev:path),*,) => {
        $(#[$attr])*
        $($vis)* enum $N {
            $($tag),*
        }
    };
    (@IMPLS, $N:ident, $from:ty, $($tag:ident => $key:literal $evdev:path),*,) => {
        impl $N {
            pub fn as_str(&self) -> &str {
                match self {
                    $($N::$tag => $key),*,
                }
            }
        }
        impl ::std::str::FromStr for $N {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($key => Ok($N::$tag)),*,
                    _ => Err(())
                }
            }
        }
        impl ::core::convert::TryFrom<$from> for $N {
            type Error = ();

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                #[allow(unreachable_patterns)]
                match value {
                    $($evdev => Ok($N::$tag)),*,
                    _ => Err(())
                }
            }
        }
        impl ::core::convert::Into<$from> for $N {
            fn into(self) -> $from {
                match self {
                    $($N::$tag => $evdev),*,
                }
            }
        }
        impl ::std::fmt::Display for $N {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                ::std::write!(f, "{}", self.as_str())
            }
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! input_enum {
    ($(#[$attr:meta])* enum $N:ident { $($t:tt)* } impl TryFrom<$from:ty>;) => {
        __input_enum_internal!($(#[$attr])* () enum $N { $($t)* } impl TryFrom<$from>;);
    };
    ($(#[$attr:meta])* pub enum $N:ident { $($t:tt)* } impl TryFrom<$from:ty>;) => {
        __input_enum_internal!($(#[$attr])* (pub) enum $N { $($t)* } impl TryFrom<$from>;);
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) enum $N:ident { $($t:tt)* } impl TryFrom<$from:ty>;) => {
        __input_enum_internal!($(#[$attr])* (pub ($($vis)+)) enum $N { $($t)* } impl TryFrom<$from>;);
    };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Input {
    Key(Key),
    Btn(Btn),
}
impl FromStr for Input {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((prefix, field)) = s.split_once(':') else {
            return Err(())
        };

        match prefix {
            "key" => Ok(Self::Key(field.parse()?)),
            "btn" => Ok(Self::Btn(field.parse()?)),
            _ => Err(()),
        }
    }
}
impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(key) => write!(f, "key:{}", key),
            Self::Btn(btn) => write!(f, "btn:{}", btn),
        }
    }
}
impl From<evdev::Key> for Input {
    fn from(value: evdev::Key) -> Self {
        if let Ok(key) = value.try_into() {
            return Self::Key(key);
        }

        if let Ok(btn) = value.try_into() {
            return Self::Btn(btn);
        }

        unreachable!();
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn from_str() {
        let a = "key:a";
        let one = "key:1";
        let north = "btn:north";

        let extra_colon = "key:z:";

        assert_eq!(Ok(Input::Key(Key::A)), a.parse());
        assert_eq!(Ok(Input::Key(Key::Key1)), one.parse());
        assert_eq!(Ok(Input::Btn(Btn::North)), north.parse());
        assert_eq!(Err(()), extra_colon.parse::<Input>());
    }

    #[test]
    fn to_string() {
        let a = Input::Key(Key::A);
        let one = Input::Key(Key::Key1);
        let north = Input::Btn(Btn::North);

        assert_eq!("key:a", &a.to_string());
        assert_eq!("key:1", &one.to_string());
        assert_eq!("btn:north", &north.to_string());
    }
}
