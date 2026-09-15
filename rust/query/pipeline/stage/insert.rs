/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::{self, Write};

use crate::{
    common::{Span, Spanned, token},
    pretty::{Pretty, indent},
    query::pipeline::stage::write_pattern::WritePattern,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Insert {
    pub span: Option<Span>,
    pub patterns: Vec<WritePattern>,
}

impl Insert {
    pub(crate) fn new(span: Option<Span>, patterns: Vec<WritePattern>) -> Self {
        Self { span, patterns }
    }
}

impl Spanned for Insert {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Pretty for Insert {
    fn fmt(&self, indent_level: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", token::Clause::Insert)?;
        for pattern in &self.patterns {
            writeln!(f)?;
            indent(indent_level, f)?;
            Pretty::fmt(pattern, indent_level, f)?;
            f.write_char(';')?;
        }
        Ok(())
    }
}

impl fmt::Display for Insert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            Pretty::fmt(self, 0, f)
        } else {
            write!(f, "{}", token::Clause::Insert)?;
            for statement in &self.patterns {
                write!(f, " {statement};")?;
            }
            Ok(())
        }
    }
}
