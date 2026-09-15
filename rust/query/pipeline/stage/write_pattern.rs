/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::{self, Write};

use crate::{
    common::{Span, Spanned, token},
    pattern::Optional,
    pretty::{Pretty, indent},
    statement::{IsSet, Statement, comparison::ComparisonStatement, thing::isa::Isa},
    util::write_joined,
    variable::Variable,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WritePattern {
    Statement(Statement),
    Optional(Optional),
    If(WritePatternIf),
}

impl Spanned for WritePattern {
    fn span(&self) -> Option<Span> {
        match self {
            Self::Statement(inner) => inner.span(),
            Self::Optional(inner) => inner.span,
            Self::If(inner) => inner.span(),
        }
    }
}

impl Pretty for WritePattern {
    fn fmt(&self, indent_level: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Statement(inner) => Pretty::fmt(inner, indent_level, f),
            Self::Optional(inner) => Pretty::fmt(inner, indent_level, f),
            Self::If(inner) => Pretty::fmt(inner, indent_level, f),
        }
    }
}

impl fmt::Display for WritePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Statement(inner) => fmt::Display::fmt(inner, f),
            Self::Optional(inner) => fmt::Display::fmt(inner, f),
            Self::If(inner) => fmt::Display::fmt(inner, f),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WritePatternIf {
    pub span: Option<Span>,
    pub conditions: Vec<WriteCondition>,
    pub patterns: Vec<WritePattern>,
}

impl WritePatternIf {
    pub fn new(span: Option<Span>, conditions: Vec<WriteCondition>, patterns: Vec<WritePattern>) -> Self {
        Self { span, conditions, patterns }
    }
}

impl Spanned for WritePatternIf {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Pretty for WritePatternIf {
    fn fmt(&self, indent_level: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {{ ", token::Keyword::If)?;
        for condition in &self.conditions {
            write!(f, "{}; ", condition)?;
        }
        writeln!(f, "}} {{")?;
        for pattern in &self.patterns {
            indent(indent_level + 1, f)?;
            Pretty::fmt(pattern, indent_level + 1, f)?;
            writeln!(f, ";")?;
        }
        indent(indent_level, f)?;
        f.write_char('}')
    }
}

impl fmt::Display for WritePatternIf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            Pretty::fmt(self, 0, f)
        } else {
            write!(f, "{} {{ ", token::Keyword::If)?;
            for condition in &self.conditions {
                write!(f, "{}; ", condition)?;
            }
            write!(f, "}} {{ ")?;
            for pattern in &self.patterns {
                write!(f, "{}; ", pattern)?;
            }
            f.write_char('}')
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WriteCondition {
    IsSet(IsSet),
    Comparison(ComparisonStatement),
    Isa { variable: Variable, isa: Isa },
}

impl Pretty for WriteCondition {}

impl fmt::Display for WriteCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsSet(isset) => {
                write!(f, "{} ", token::Keyword::IsSet)?;
                write_joined!(f, ", ", isset.variables)
            }
            Self::Comparison(cmp) => write!(f, "{}", cmp),
            Self::Isa { variable, isa } => write!(f, "{} {}", variable, isa),
        }
    }
}
