/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[macro_export]
macro_rules! error_messages {
    {
        $name:ident code: $code_pfx:literal, type: $message_pfx:literal,
        $($error_name:ident $({ $($field:ident : $inner:ty),+ $(,)? })? = $code:literal: $body:literal),+ $(,)?
    } => {
        #[derive(Clone, Eq, PartialEq)]
        pub enum $name {$(
            $error_name$( { $($field: $inner),+ })?,
        )*}

        impl $name {
            pub const PREFIX: &'static str = $code_pfx;

            pub const fn code(&self) -> usize {
                match self {$(
                    Self::$error_name $({ $($field: _),+ })? => $code,
                )*}
            }

            pub fn format_code(&self) -> String {
                format!(concat!("[", $code_pfx, "{}{}]"), self.padding(), self.code())
            }

            pub fn message(&self) -> String {
                match self {$(
                    Self::$error_name $({$($field),+})? => format!($body $($(, $field = $field)+)?),
                )*}
            }

            const fn max_code() -> usize {
                let mut max = usize::MIN;
                $(max = if $code > max { $code } else { max };)*
                max
            }

            const fn num_digits(x: usize) -> usize {
                if (x < 10) { 1 } else { 1 + Self::num_digits(x/10) }
            }

            const fn padding(&self) -> &'static str {
                match Self::num_digits(Self::max_code()) - Self::num_digits(self.code()) {
                    0 => "",
                    1 => "0",
                    2 => "00",
                    3 => "000",
                    _ => unreachable!(),
                }
            }

            const fn name(&self) -> &'static str {
                match self {$(
                    Self::$error_name $({ $($field: _),+ })? => concat!(stringify!($name), "::", stringify!($error_name)),
                )*}
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    concat!("[", $code_pfx, "{}{}] ", $message_pfx, ": {}"),
                    self.padding(),
                    self.code(),
                    self.message()
                )
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut debug_struct = f.debug_struct(self.name());
                debug_struct.field("message", &format!("{}", self));
                $(
                    $(
                        if let Self::$error_name { $($field),+ } = &self {
                            $(debug_struct.field(stringify!($field), &$field);)+
                        }
                    )?
                )*
                debug_struct.finish()
            }
        }

        impl std::error::Error for $name {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                None
            }
        }
    };
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    error_messages! { TestError
        code: "TST", type: "Test Error",
        BasicError =
            1: "This is a basic error.",
        ErrorWithAttributes { int: i32, string: String } =
            2: "This is an error with i32 {int} and string '{string}'.",
        MultiLine =
            3: "This is an error,\nthat spans,\nmultiple lines."
    }

    #[test]
    pub fn debug_includes_display() {
        let errors = [
            TestError::BasicError,
            TestError::ErrorWithAttributes { int: 1, string: "error message".to_string() },
            TestError::MultiLine,
        ];

        for error in errors {
            let display = format!("{error}");
            let compact_debug = format!("{error:?}");
            let expanded_debug = format!("{error:#?}");
            assert!(compact_debug.contains(&display.replace('\n', "\\n")));
            assert!(expanded_debug.contains(&display.replace('\n', "\\n")));
        }
    }
}
