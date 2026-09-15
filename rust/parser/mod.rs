/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use pest::Parser;
use pest_derive::Parser;

use self::{
    define::{function::visit_definition_function, struct_::visit_definition_struct, visit_query_define},
    type_::visit_label,
    undefine::visit_query_undefine,
};
use crate::{
    Result,
    common::{
        Span, Spanned,
        error::{TypeQLError, syntax_error},
        identifier::Identifier,
        token,
    },
    parser::{literal::visit_value_literal, pipeline::visit_query_pipeline_preambled, redefine::visit_query_redefine},
    query::{Query, QueryStructure, SchemaQuery},
    schema::definable,
    type_::Label,
    value::ValueLiteral,
    variable::{Optional, Variable},
};

mod annotation;
mod define;
mod expression;
mod literal;
mod pipeline;
mod redefine;
mod statement;
mod type_;
mod undefine;

#[cfg(test)]
mod test;

#[derive(Parser)]
#[grammar = "parser/typeql.pest"]
pub(crate) struct TypeQLParser;

type Node<'a> = pest::iterators::Pair<'a, Rule>;
type ChildNodes<'a> = pest::iterators::Pairs<'a, Rule>;

impl Spanned for Node<'_> {
    fn span(&self) -> Option<Span> {
        let begin_offset = self.as_span().start_pos().pos();
        let end_offset = self.as_span().end_pos().pos();
        Some(Span { begin_offset, end_offset })
    }
}

trait IntoChildNodes<'a> {
    fn into_child(self) -> Node<'a>;
    fn into_children(self) -> ChildNodes<'a>;
}

impl<'a> IntoChildNodes<'a> for Node<'a> {
    fn into_child(self) -> Node<'a> {
        let mut children = self.into_children();
        let child = children.consume_any();
        match children.try_consume_any() {
            None => child,
            Some(next) => unreachable!("{child} is followed by more tokens: {next}"),
        }
    }

    fn into_children(self) -> ChildNodes<'a> {
        self.into_inner()
    }
}

trait RuleMatcher<'a> {
    fn skip_expected(&mut self, rule: Rule) -> &mut Self;
    fn consume_expected(&mut self, rule: Rule) -> Node<'a>;
    fn peek_rule(&mut self) -> Option<Rule>;
    fn try_consume_expected(&mut self, rule: Rule) -> Option<Node<'a>>;
    fn consume_any(&mut self) -> Node<'a>;
    fn try_consume_any(&mut self) -> Option<Node<'a>>;
}

impl<'a, T: Iterator<Item = Node<'a>> + Clone> RuleMatcher<'a> for T {
    fn skip_expected(&mut self, rule: Rule) -> &mut Self {
        self.consume_expected(rule);
        self
    }

    fn consume_expected(&mut self, _rule: Rule) -> Node<'a> {
        let next = self.consume_any();
        assert_eq!(next.as_rule(), _rule);
        next
    }

    fn consume_any(&mut self) -> Node<'a> {
        self.next().expect("attempting to consume from an empty iterator")
    }

    fn peek_rule(&mut self) -> Option<Rule> {
        self.clone().peekable().peek().map(|node| node.as_rule())
    }

    fn try_consume_expected(&mut self, rule: Rule) -> Option<Node<'a>> {
        (Some(rule) == self.peek_rule()).then(|| self.consume_any())
    }

    fn try_consume_any(&mut self) -> Option<Node<'a>> {
        self.next()
    }
}

fn parse(rule: Rule, string: &str) -> Result<ChildNodes<'_>> {
    let result = TypeQLParser::parse(rule, string);
    match result {
        Ok(nodes) => Ok(nodes),
        Err(error) => Err(syntax_error(string, error).into()),
    }
}

fn parse_single(rule: Rule, string: &str) -> Result<Node<'_>> {
    Ok(parse(rule, string)?.consume_any())
}

pub(crate) fn visit_eof_query(query: &str) -> Result<Query> {
    Ok(visit_query(parse_single(Rule::eof_query, query)?.into_children().consume_expected(Rule::query)))
}

pub(crate) fn visit_query_prefix(query: &str) -> Result<Query> {
    Ok(visit_query(parse_single(Rule::query_prefix, query)?.into_child()))
}

pub(crate) fn visit_eof_definition_function(query: &str) -> Result<definable::Function> {
    Ok(visit_definition_function(
        parse_single(Rule::eof_definition_function, query)?.into_children().consume_expected(Rule::definition_function),
    ))
}

pub(crate) fn visit_eof_definition_struct(query: &str) -> Result<definable::Struct> {
    Ok(visit_definition_struct(
        parse_single(Rule::eof_definition_struct, query)?.into_children().consume_expected(Rule::definition_struct),
    ))
}

pub(crate) fn visit_eof_label(label: &str) -> Result<Label> {
    let parsed = parse_single(Rule::eof_label, label)?.into_children().consume_expected(Rule::label);
    let string = parsed.as_str();
    if string != label {
        Err(TypeQLError::InvalidTypeLabel { label: label.to_string() })?;
    }
    Ok(visit_label(parsed))
}

pub(crate) fn visit_eof_value(value: &str) -> Result<ValueLiteral> {
    Ok(visit_value_literal(parse_single(Rule::eof_value, value)?.into_children().consume_expected(Rule::value_literal))
        .inner)
}

fn visit_query(node: Node<'_>) -> Query {
    debug_assert_eq!(node.as_rule(), Rule::query);
    let span = node.span();
    let mut children = node.into_children();
    let query_structure = visit_query_structure(children.consume_expected(Rule::query_structure));
    let query = match children.try_consume_expected(Rule::query_end) {
        None => Query::new(span, query_structure, false),
        Some(_) => Query::new(span, query_structure, true),
    };
    debug_assert_eq!(children.try_consume_any(), None);
    query
}

fn visit_query_structure(node: Node<'_>) -> QueryStructure {
    debug_assert_eq!(node.as_rule(), Rule::query_structure);
    let mut children = node.into_children();
    let child = children.consume_any();
    match child.as_rule() {
        Rule::query_schema => QueryStructure::Schema(visit_query_schema(child)),
        Rule::query_pipeline_preambled => QueryStructure::Pipeline(visit_query_pipeline_preambled(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_query_schema(node: Node<'_>) -> SchemaQuery {
    debug_assert_eq!(node.as_rule(), Rule::query_schema);
    let mut children = node.into_children();
    let child = children.consume_any();
    let query = match child.as_rule() {
        Rule::query_define => SchemaQuery::Define(visit_query_define(child)),
        Rule::query_redefine => SchemaQuery::Redefine(visit_query_redefine(child)),
        Rule::query_undefine => SchemaQuery::Undefine(visit_query_undefine(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    };
    debug_assert_eq!(children.try_consume_any(), None);
    query
}

fn visit_kind(node: Node<'_>) -> token::Kind {
    debug_assert_eq!(node.as_rule(), Rule::kind);
    let child = node.into_child();
    match child.as_rule() {
        Rule::ENTITY => token::Kind::Entity,
        Rule::RELATION => token::Kind::Relation,
        Rule::ATTRIBUTE => token::Kind::Attribute,
        Rule::ROLE => token::Kind::Role,
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_identifier(node: Node<'_>) -> Identifier {
    debug_assert_eq!(node.as_rule(), Rule::identifier);
    Identifier::new(node.span(), node.as_str().to_owned())
}

fn visit_var(node: Node<'_>) -> Variable {
    debug_assert_eq!(node.as_rule(), Rule::var);
    let span = node.span();
    let child = node.into_child();
    match child.as_rule() {
        Rule::VAR_ANONYMOUS => Variable::Anonymous { span, optional: None },
        Rule::var_named => visit_var_named(child),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_var_or_optional(node: Node<'_>) -> Variable {
    debug_assert_eq!(node.as_rule(), Rule::var_or_optional);
    let child = node.into_child();
    match child.as_rule() {
        Rule::var => visit_var(child),
        Rule::var_optional => visit_var_optional(child),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_var_optional(node: Node<'_>) -> Variable {
    debug_assert_eq!(node.as_rule(), Rule::var_optional);
    let span = node.span();
    match visit_var(node.into_child()) {
        Variable::Anonymous { .. } => Variable::Anonymous { span, optional: Some(Optional) },
        Variable::Named { ident, .. } => Variable::Named { span, ident, optional: Some(Optional) },
    }
}

fn visit_var_named(node: Node<'_>) -> Variable {
    debug_assert_eq!(node.as_rule(), Rule::var_named);
    let span = node.span();
    Variable::Named { span, ident: visit_identifier_var(node.into_child()), optional: None }
}

fn visit_identifier_var(node: Node<'_>) -> Identifier {
    debug_assert_eq!(node.as_rule(), Rule::identifier_var);
    Identifier::new(node.span(), node.as_str().to_owned())
}

fn visit_vars(node: Node<'_>) -> Vec<Variable> {
    debug_assert_eq!(node.as_rule(), Rule::vars);
    node.into_children().map(visit_var).collect()
}

fn visit_vars_assignment(node: Node<'_>) -> Vec<Variable> {
    debug_assert_eq!(node.as_rule(), Rule::vars_assignment);
    node.into_children().map(visit_var_or_optional).collect()
}

#[cfg(test)]
pub mod tests {
    pub fn is_keyword(identifier: &str) -> bool {
        super::parse(super::Rule::reserved, identifier).is_ok()
    }
}
