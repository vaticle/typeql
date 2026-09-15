/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{collections::HashMap, fmt, fmt::Write};

use self::comparison::ComparisonStatement;
pub use self::{thing::Thing, type_::Type};
use crate::{
    common::{Span, Spanned, identifier::Identifier, token},
    expression::Expression,
    pretty::{Pretty, indent},
    util::write_joined,
    variable::Variable,
};

pub mod comparison;
pub mod thing;
pub mod type_;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Is {
    pub span: Option<Span>,
    pub lhs: Variable,
    pub rhs: Variable,
}

impl Is {
    pub(crate) fn new(span: Option<Span>, lhs: Variable, rhs: Variable) -> Self {
        Self { span, lhs, rhs }
    }
}

impl Spanned for Is {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Pretty for Is {}

impl fmt::Display for Is {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.lhs, token::Keyword::Is, self.rhs)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InIterable {
    pub span: Option<Span>,
    pub lhs: Vec<Variable>,
    pub rhs: Expression,
}

impl InIterable {
    pub(crate) fn new(span: Option<Span>, lhs: Vec<Variable>, rhs: Expression) -> Self {
        Self { span, lhs, rhs }
    }
}

impl Spanned for InIterable {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Pretty for InIterable {}

impl fmt::Display for InIterable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", token::Keyword::Let)?;
        write_joined!(f, ", ", self.lhs)?;
        write!(f, " {} {}", token::Keyword::In, self.rhs)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DeconstructField {
    Variable(Variable),
    Deconstruct(StructDeconstruct),
}

impl Pretty for DeconstructField {
    fn fmt(&self, indent_level: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeconstructField::Variable(var) => Pretty::fmt(var, indent_level, f),
            DeconstructField::Deconstruct(struct_deconstruct) => Pretty::fmt(struct_deconstruct, indent_level, f),
        }
    }
}

impl fmt::Display for DeconstructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeconstructField::Variable(var) => fmt::Display::fmt(var, f),
            DeconstructField::Deconstruct(struct_deconstruct) => fmt::Display::fmt(struct_deconstruct, f),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructDeconstruct {
    pub span: Option<Span>,
    field_map: HashMap<Identifier, DeconstructField>,
}

impl StructDeconstruct {
    pub fn new(span: Option<Span>, field_map: HashMap<Identifier, DeconstructField>) -> Self {
        Self { span, field_map }
    }
}

impl Spanned for StructDeconstruct {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Pretty for StructDeconstruct {
    fn fmt(&self, indent_level: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        indent(indent_level, f)?;
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for StructDeconstruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", token::Char::CurlyLeft)?;
        for (identifier, field_deconstruct) in &self.field_map {
            write!(f, "{}{} {},", identifier, token::Char::Colon, field_deconstruct)?;
        }
        write!(f, "{}", token::Char::CurlyRight)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AssignmentPattern {
    Variables(Vec<Variable>),
    Deconstruct(StructDeconstruct),
}

impl Pretty for AssignmentPattern {}

impl fmt::Display for AssignmentPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variables(vars) => write_joined!(f, ", ", vars),
            Self::Deconstruct(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Assignment {
    pub span: Option<Span>,
    pub lhs: AssignmentPattern,
    pub rhs: Expression,
}

impl Assignment {
    pub fn new(span: Option<Span>, lhs: AssignmentPattern, rhs: Expression) -> Self {
        Self { span, lhs, rhs }
    }
}

impl Spanned for Assignment {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Pretty for Assignment {}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} = {}", token::Keyword::Let, self.lhs, self.rhs)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IsSet {
    pub span: Option<Span>,
    pub variables: Vec<Variable>,
}

impl IsSet {
    pub fn new(span: Option<Span>, variables: Vec<Variable>) -> Self {
        Self { span, variables }
    }
}

impl Spanned for IsSet {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Pretty for IsSet {}

impl fmt::Display for IsSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", token::Keyword::IsSet)?;
        write_joined!(f, ", ", self.variables)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement {
    Is(Is),
    InIterable(InIterable),
    Comparison(ComparisonStatement),
    Assignment(Assignment),
    IsSet(IsSet),
    Thing(Thing),
    Type(Type),
}

impl Spanned for Statement {
    fn span(&self) -> Option<Span> {
        match self {
            Statement::Is(inner) => inner.span(),
            Statement::InIterable(inner) => inner.span(),
            Statement::Comparison(inner) => inner.span(),
            Statement::Assignment(inner) => inner.span(),
            Statement::IsSet(inner) => inner.span(),
            Statement::Thing(inner) => inner.span(),
            Statement::Type(inner) => inner.span(),
        }
    }
}

impl Pretty for Statement {
    fn fmt(&self, indent_level: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Is(inner) => Pretty::fmt(inner, indent_level, f),
            Statement::InIterable(inner) => Pretty::fmt(inner, indent_level, f),
            Statement::Comparison(inner) => Pretty::fmt(inner, indent_level, f),
            Statement::Assignment(inner) => Pretty::fmt(inner, indent_level, f),
            Statement::IsSet(inner) => Pretty::fmt(inner, indent_level, f),
            Statement::Thing(inner) => Pretty::fmt(inner, indent_level, f),
            Statement::Type(inner) => Pretty::fmt(inner, indent_level, f),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Is(inner) => fmt::Display::fmt(inner, f),
            Statement::InIterable(inner) => fmt::Display::fmt(inner, f),
            Statement::Comparison(inner) => fmt::Display::fmt(inner, f),
            Statement::Assignment(inner) => fmt::Display::fmt(inner, f),
            Statement::IsSet(inner) => fmt::Display::fmt(inner, f),
            Statement::Thing(inner) => fmt::Display::fmt(inner, f),
            Statement::Type(inner) => fmt::Display::fmt(inner, f),
        }
    }
}
