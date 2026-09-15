/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt;

macro_rules! string_enum {
    {$name:ident $($item:ident = $value:tt),* $(,)?} => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
        pub enum $name {
            $($item),*
        }

        impl $name {
            pub const NAMES : &'static [&'static str] = &[$($value), *];

            pub const fn as_str(&self) -> &'static str {
               match self {
                    $($name::$item => $value,)*
                }
            }
        }

        impl From<&str> for $name {
            fn from(string: &str) -> Self {
                match string {
                    $($value => $name::$item,)*
                    _ => panic!("Unexpected input while parsing {}: '{}'", stringify!($name), string),
                }
            }
        }

        impl From<String> for $name {
            fn from(string: String) -> Self {
                Self::from(string.as_str())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    }
}

string_enum! { Kind
    Entity = "entity",
    Relation = "relation",
    Attribute = "attribute",
    Role = "role",
}

string_enum! { Clause
    Define = "define",
    Redefine = "redefine",
    Undefine = "undefine",
    Insert = "insert",
    Put = "put",
    Update = "update",
    Delete = "delete",
    Match = "match",
    Fetch = "fetch",
    With = "with",
    End = "end",
}

string_enum! { Operator
    Select = "select",
    Sort = "sort",
    Offset = "offset",
    Limit = "limit",
    Reduce = "reduce",
    Require = "require",
    Distinct = "distinct",
}

string_enum! { LogicOperator
    And = "and",
    Or = "or",
    Not = "not",
}

string_enum! { Comparator
    // equality
    Eq = "==",
    EqLegacy = "=",   // TODO: Deprecate '=' as equality in 3.0
    Neq = "!=",
    Gt = ">",
    Gte = ">=",
    Lt = "<",
    Lte = "<=",
    // substring
    Contains = "contains",
    Like = "like",
}

impl Comparator {
    pub fn is_equality(&self) -> bool {
        use Comparator::*;
        matches!(self, Eq | EqLegacy | Neq | Gt | Gte | Lt | Lte) // TODO: Deprecate '=' as equality in 3.0
    }

    pub fn is_substring(&self) -> bool {
        use Comparator::*;
        matches!(self, Contains | Like)
    }
}

string_enum! { Keyword
    Abstract = "abstract",
    As = "as",
    Alias = "alias",
    Assign = "=",
    Check = "check",
    First = "first",
    From = "from",
    Fun = "fun",
    Given = "given",
    Has = "has",
    IID = "iid",
    In = "in",
    Is = "is",
    Isa = "isa",
    IsaX = "isa!",
    IsSet = "isset",
    Label = "label",
    Last = "last",
    Let = "let",
    Links = "links",
    Not = "not",
    Of = "of",
    Or = "or",
    Owns = "owns",
    Plays = "plays",
    Relates = "relates",
    Return = "return",
    Struct = "struct",
    Sub = "sub",
    SubX = "sub!",
    Try = "try",
    Value = "value",
    Groupby = "groupby",
}

string_enum! { Annotation
    Abstract = "abstract",
    Cardinality = "card",
    Cascade = "cascade",
    Distinct = "distinct",
    Doc = "doc",
    Independent = "independent",
    Key = "key",
    Meta = "meta",
    Range = "range",
    Regex = "regex",
    Subkey = "subkey",
    Unique = "unique",
    Values = "values",
}

string_enum! { ReduceOperatorCount
    Count = "count",
}

string_enum! { ReduceOperatorStat
    Max = "max",
    Mean = "mean",
    Median = "median",
    Min = "min",
    Std = "std",
    Sum = "sum",
}

string_enum! { ReduceOperatorCollect
    List = "list",
}

string_enum! { ValueType
    Boolean = "boolean",
    Date = "date",
    DateTime = "datetime",
    DateTimeTZ = "datetime-tz",
    Decimal = "decimal",
    Double = "double",
    Duration = "duration",
    Integer = "integer",
    String = "string",
}

string_enum! { Order
    Asc = "asc",
    Desc = "desc",
}

string_enum! { ArithmeticOperator
    Add = "+",
    Subtract = "-",
    Multiply = "*",
    Divide = "/",
    Modulo = "%",
    Power = "^",
}

string_enum! { Function
    Abs = "abs",
    Ceil = "ceil",
    Floor = "floor",
    Iid = "iid",
    Label = "label",
    Len = "len",
    Max = "max",
    Min = "min",
    Round = "round",
}

string_enum! { Char
    Question = "?",
    Dollar = "$",
    Underscore = "_",
    Comma = ",",
    Dot = ".",
    Star = "*",
    Semicolon = ";",
    Colon = ":",
    Hash = "#",
    ParenLeft = "(",
    ParenRight = ")",
    CurlyLeft = "{",
    CurlyRight = "}",
    SquareLeft = "[",
    SquareRight = "]",
}

string_enum! { BooleanValue
    True = "true",
    False = "false",
}
