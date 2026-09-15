/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt;

pub use self::{
    delete::Delete, fetch::Fetch, given::Given, insert::Insert, match_::Match, modifier::Operator, put::Put,
    reduce::Reduce, update::Update,
};
use crate::{
    common::{Span, Spanned},
    pretty::Pretty,
    util::enum_getter,
};

pub mod delete;
pub mod fetch;
pub mod given;
mod insert;
mod match_;
pub mod modifier;
mod put;
pub mod reduce;
mod update;
pub mod write_pattern;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Stage {
    Given(Given),
    Match(Match),
    Insert(Insert),
    Put(Put),
    Update(Update),
    Fetch(Fetch),
    Delete(Delete),
    Operator(Operator),
}

enum_getter! { Stage
    into_given(Given) => Given,
    into_match(Match) => Match,
    into_insert(Insert) => Insert,
    into_put(Put) => Put,
    into_update(Update) => Update,
    into_fetch(Fetch) => Fetch,
    into_delete(Delete) => Delete,
    into_modifier(Operator) => Operator,
}

impl Spanned for Stage {
    fn span(&self) -> Option<Span> {
        match self {
            Self::Given(inner) => inner.span(),
            Self::Match(inner) => inner.span(),
            Self::Insert(inner) => inner.span(),
            Self::Put(inner) => inner.span(),
            Self::Update(inner) => inner.span(),
            Self::Fetch(inner) => inner.span(),
            Self::Delete(inner) => inner.span(),
            Self::Operator(inner) => inner.span(),
        }
    }
}

impl Pretty for Stage {
    fn fmt(&self, indent_level: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Given(inner) => Pretty::fmt(inner, indent_level, f),
            Self::Match(inner) => Pretty::fmt(inner, indent_level, f),
            Self::Insert(inner) => Pretty::fmt(inner, indent_level, f),
            Self::Put(inner) => Pretty::fmt(inner, indent_level, f),
            Self::Update(inner) => Pretty::fmt(inner, indent_level, f),
            Self::Fetch(inner) => Pretty::fmt(inner, indent_level, f),
            Self::Delete(inner) => Pretty::fmt(inner, indent_level, f),
            Self::Operator(inner) => Pretty::fmt(inner, indent_level, f),
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Given(inner) => fmt::Display::fmt(inner, f),
            Self::Match(inner) => fmt::Display::fmt(inner, f),
            Self::Insert(inner) => fmt::Display::fmt(inner, f),
            Self::Put(inner) => fmt::Display::fmt(inner, f),
            Self::Update(inner) => fmt::Display::fmt(inner, f),
            Self::Fetch(inner) => fmt::Display::fmt(inner, f),
            Self::Delete(inner) => fmt::Display::fmt(inner, f),
            Self::Operator(inner) => fmt::Display::fmt(inner, f),
        }
    }
}
