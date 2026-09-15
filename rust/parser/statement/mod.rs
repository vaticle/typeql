/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use self::{
    single::{
        visit_statement_assignment, visit_statement_comparison, visit_statement_in, visit_statement_is,
        visit_statement_isset,
    },
    thing::{visit_statement_thing_basic, visit_statement_thing_relation_anonymous},
    type_::visit_statement_type,
};
use super::{IntoChildNodes, Node, Rule, RuleMatcher, expression::visit_expression_value};
use crate::{
    common::{Spanned, error::TypeQLError, token::Comparator},
    statement::{Statement, comparison::Comparison},
};

pub(super) mod single;
pub(super) mod thing;
pub(super) mod type_;

pub(super) fn visit_statement(node: Node<'_>) -> Statement {
    debug_assert_eq!(node.as_rule(), Rule::statement);
    let child = node.into_child();
    match child.as_rule() {
        Rule::statement_thing_basic => visit_statement_thing_basic(child),
        Rule::statement_thing_relation_anonymous => visit_statement_thing_relation_anonymous(child),
        Rule::statement_type => visit_statement_type(child),
        Rule::statement_is => Statement::Is(visit_statement_is(child)),
        Rule::statement_in => Statement::InIterable(visit_statement_in(child)),
        Rule::statement_comparison => Statement::Comparison(visit_statement_comparison(child)),
        Rule::statement_assignment => Statement::Assignment(visit_statement_assignment(child)),
        Rule::statement_isset => Statement::IsSet(visit_statement_isset(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_comparison(node: Node<'_>) -> Comparison {
    debug_assert_eq!(node.as_rule(), Rule::comparison);
    let span = node.span();
    let mut children = node.into_children();
    let comparator_node = children.consume_expected(Rule::comparator).into_child();
    let comparator = match comparator_node.as_rule() {
        Rule::EQ => Comparator::Eq,
        Rule::NEQ => Comparator::Neq,
        Rule::GTE => Comparator::Gte,
        Rule::GT => Comparator::Gt,
        Rule::LTE => Comparator::Lte,
        Rule::LT => Comparator::Lt,
        Rule::CONTAINS => Comparator::Contains,
        Rule::LIKE => Comparator::Like,
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: comparator_node.as_str().to_owned() }),
    };
    let rhs = visit_expression_value(children.consume_expected(Rule::expression_value));
    debug_assert_eq!(children.try_consume_any(), None);
    Comparison::new(span, comparator, rhs)
}
