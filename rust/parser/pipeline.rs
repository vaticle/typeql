/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use itertools::Itertools;

use super::{
    IntoChildNodes, Node, Rule, RuleMatcher,
    define::function::visit_definition_function,
    expression::{visit_expression, visit_expression_function},
    literal::{visit_integer_literal, visit_quoted_string_literal},
    statement::{thing::visit_relation, visit_statement},
    type_::{visit_label, visit_label_list},
    visit_var, visit_var_or_optional, visit_vars,
};
use crate::{
    TypeRef, TypeRefAny,
    common::{
        Spanned,
        error::TypeQLError,
        token::{Order, ReduceOperatorCollect, ReduceOperatorStat},
    },
    parser::{
        define::function::{visit_function_block, visit_pipeline_arguments},
        statement::{
            single::{visit_statement_comparison, visit_statement_isset},
            thing::visit_isa_constraint,
        },
    },
    pattern::{Conjunction, Disjunction, Negation, Optional, Pattern},
    query::{
        Pipeline,
        pipeline::{
            Preamble,
            stage::{
                Delete, Fetch, Insert, Match, Operator, Put, Reduce, Stage, Update,
                delete::{Deletable, DeletableKind},
                fetch::FetchSome,
                given::Given,
                modifier::{Distinct, Limit, Offset, OrderedVariable, Require, Select, Sort},
                reduce::{Collect, Count, Reducer, Stat},
                write_pattern::{WriteCondition, WritePattern, WritePatternIf},
            },
        },
        stage::{
            fetch::{
                FetchAttribute, FetchList, FetchObject, FetchObjectBody, FetchObjectEntry, FetchSingle, FetchStream,
            },
            reduce::ReduceAssign,
        },
    },
    value::StringLiteral,
};

pub(super) fn visit_query_pipeline_preambled(node: Node<'_>) -> Pipeline {
    debug_assert_eq!(node.as_rule(), Rule::query_pipeline_preambled);
    let span = node.span();
    let mut children = node.into_children();

    let preambles = children.take_while_ref(|child| child.as_rule() == Rule::preamble).map(visit_preamble).collect();
    let stages = visit_query_pipeline(children.consume_expected(Rule::query_pipeline));
    debug_assert_eq!(children.try_consume_any(), None);

    Pipeline::new(span, preambles, stages)
}

fn visit_query_pipeline(node: Node<'_>) -> Vec<Stage> {
    debug_assert_eq!(node.as_rule(), Rule::query_pipeline);
    let mut children = node.into_children();
    let mut stages = Vec::new();
    stages.extend(children.try_consume_expected(Rule::query_stage_given).map(visit_query_stage_given));
    stages.extend(
        children.take_while_ref(|child| child.as_rule() == Rule::query_stage).map(visit_query_stage).collect_vec(),
    );
    stages.extend(children.try_consume_expected(Rule::query_stage_terminal).map(visit_query_stage_terminal));
    debug_assert_eq!(children.try_consume_any(), None);
    stages
}

fn visit_query_stage_given(node: Node<'_>) -> Stage {
    debug_assert_eq!(node.as_rule(), Rule::query_stage_given);
    let span = node.span();
    let mut children = node.into_children();
    let variables =
        visit_pipeline_arguments(children.skip_expected(Rule::GIVEN).consume_expected(Rule::pipeline_arguments));
    debug_assert_eq!(children.try_consume_any(), None);
    Stage::Given(Given::new(span, variables))
}

fn visit_preamble(node: Node<'_>) -> Preamble {
    debug_assert_eq!(node.as_rule(), Rule::preamble);
    let span = node.span();
    let mut children = node.into_children();
    let patterns =
        visit_definition_function(children.skip_expected(Rule::WITH).consume_expected(Rule::definition_function));
    debug_assert_eq!(children.try_consume_any(), None);
    Preamble::new(span, patterns)
}

pub(super) fn visit_query_stage(node: Node<'_>) -> Stage {
    debug_assert_eq!(node.as_rule(), Rule::query_stage);
    let child = node.into_child();
    match child.as_rule() {
        Rule::clause_match => Stage::Match(visit_clause_match(child)),
        Rule::clause_insert => Stage::Insert(visit_clause_insert(child)),
        Rule::clause_put => Stage::Put(visit_clause_put(child)),
        Rule::clause_update => Stage::Update(visit_clause_update(child)),
        Rule::clause_delete => Stage::Delete(visit_clause_delete(child)),
        Rule::operator_stream => Stage::Operator(visit_operator_stream(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_query_stage_terminal(node: Node<'_>) -> Stage {
    debug_assert_eq!(node.as_rule(), Rule::query_stage_terminal);
    let child = node.into_child();
    match child.as_rule() {
        Rule::clause_fetch => Stage::Fetch(visit_clause_fetch(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

pub(super) fn visit_clause_match(node: Node<'_>) -> Match {
    debug_assert_eq!(node.as_rule(), Rule::clause_match);
    let span = node.span();
    let mut children = node.into_children();
    let patterns = visit_patterns(children.skip_expected(Rule::MATCH).consume_expected(Rule::patterns));
    debug_assert_eq!(children.try_consume_any(), None);
    Match::new(span, patterns)
}

pub(super) fn visit_patterns(node: Node<'_>) -> Vec<Pattern> {
    debug_assert_eq!(node.as_rule(), Rule::patterns);
    node.into_children().map(visit_pattern).collect()
}

pub(super) fn visit_pattern(node: Node<'_>) -> Pattern {
    debug_assert_eq!(node.as_rule(), Rule::pattern);
    let child = node.into_child();
    match child.as_rule() {
        Rule::pattern_conjunction => Pattern::Conjunction(visit_pattern_conjunction(child)),
        Rule::pattern_disjunction => Pattern::Disjunction(visit_pattern_disjunction(child)),
        Rule::pattern_negation => Pattern::Negation(visit_pattern_negation(child)),
        Rule::pattern_try => Pattern::Optional(visit_pattern_try(child)),
        Rule::statement => Pattern::Statement(visit_statement(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_pattern_conjunction(node: Node<'_>) -> Conjunction {
    debug_assert_eq!(node.as_rule(), Rule::pattern_conjunction);
    let span = node.span();
    Conjunction::new(span, visit_patterns(node.into_child()))
}

fn visit_pattern_disjunction(node: Node<'_>) -> Disjunction {
    debug_assert_eq!(node.as_rule(), Rule::pattern_disjunction);
    let span = node.span();
    let mut branches = Vec::new();
    let mut children = node.into_children();
    while let Some(branch) = children.try_consume_expected(Rule::patterns) {
        branches.push(visit_patterns(branch));
        children.try_consume_expected(Rule::OR);
    }
    debug_assert_eq!(children.try_consume_any(), None);
    Disjunction::new(span, branches)
}

fn visit_pattern_negation(node: Node<'_>) -> Negation {
    debug_assert_eq!(node.as_rule(), Rule::pattern_negation);
    let span = node.span();
    let mut children = node.into_children();
    let patterns = children.skip_expected(Rule::NOT).consume_expected(Rule::patterns);
    debug_assert_eq!(children.try_consume_any(), None);
    Negation::new(span, visit_patterns(patterns))
}

fn visit_pattern_try(node: Node<'_>) -> Optional {
    debug_assert_eq!(node.as_rule(), Rule::pattern_try);
    let span = node.span();
    let mut children = node.into_children();
    let patterns = children.skip_expected(Rule::TRY).consume_expected(Rule::patterns);
    debug_assert_eq!(children.try_consume_any(), None);
    Optional::new(span, visit_patterns(patterns))
}

fn visit_clause_insert(node: Node<'_>) -> Insert {
    debug_assert_eq!(node.as_rule(), Rule::clause_insert);
    let span = node.span();
    let mut children = node.into_children();
    let patterns = visit_write_patterns(children.skip_expected(Rule::INSERT).consume_expected(Rule::write_patterns));
    debug_assert_eq!(children.try_consume_any(), None);
    Insert::new(span, patterns)
}

fn visit_clause_put(node: Node<'_>) -> Put {
    debug_assert_eq!(node.as_rule(), Rule::clause_put);
    let span = node.span();
    let mut children = node.into_children();
    let patterns = visit_write_patterns(children.skip_expected(Rule::PUT).consume_expected(Rule::write_patterns));
    debug_assert_eq!(children.try_consume_any(), None);
    Put::new(span, patterns)
}

fn visit_clause_update(node: Node<'_>) -> Update {
    debug_assert_eq!(node.as_rule(), Rule::clause_update);
    let span = node.span();
    let mut children = node.into_children();
    let patterns = visit_write_patterns(children.skip_expected(Rule::UPDATE).consume_expected(Rule::write_patterns));
    debug_assert_eq!(children.try_consume_any(), None);
    Update::new(span, patterns)
}

fn visit_write_patterns(node: Node<'_>) -> Vec<WritePattern> {
    debug_assert_eq!(node.as_rule(), Rule::write_patterns);
    node.into_children().map(visit_write_pattern).collect()
}

fn visit_write_pattern(node: Node<'_>) -> WritePattern {
    debug_assert_eq!(node.as_rule(), Rule::write_pattern);
    let child = node.into_child();
    match child.as_rule() {
        Rule::statement => WritePattern::Statement(visit_statement(child)),
        Rule::pattern_try => WritePattern::Optional(visit_pattern_try(child)),
        Rule::write_pattern_if => visit_write_pattern_if(child),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_write_pattern_if(node: Node<'_>) -> WritePattern {
    debug_assert_eq!(node.as_rule(), Rule::write_pattern_if);
    let span = node.span();
    let mut children = node.into_children();
    let conditions = visit_write_if_clause(children.consume_expected(Rule::write_if_clause));
    let patterns = visit_write_patterns(children.consume_expected(Rule::write_patterns));
    WritePattern::If(WritePatternIf::new(span, conditions, patterns))
}

fn visit_clause_delete(node: Node<'_>) -> Delete {
    debug_assert_eq!(node.as_rule(), Rule::clause_delete);
    let span = node.span();
    let deletables = node
        .into_children()
        .skip_expected(Rule::DELETE)
        .map(|child| match child.as_rule() {
            Rule::statement_deletable => visit_statement_deletable(child),
            Rule::pattern_try_deletable => visit_pattern_try_deletable(child),
            Rule::pattern_if_deletable => visit_pattern_if_deletable(child),
            _ => unreachable!(
                "Unrecognised statement inside delete clause: {:?}",
                TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }
            ),
        })
        .collect();
    Delete::new(span, deletables)
}

fn visit_statement_deletable(node: Node<'_>) -> Deletable {
    debug_assert_eq!(node.as_rule(), Rule::statement_deletable);
    let span = node.span();
    let mut children = node.into_children();
    let kind = match children.peek_rule().unwrap() {
        Rule::var if children.len() == 1 => {
            DeletableKind::Concept { variable: visit_var(children.consume_expected(Rule::var)) }
        }
        Rule::HAS | Rule::var => {
            children.try_consume_expected(Rule::HAS);
            let attribute = visit_var(children.consume_expected(Rule::var));
            children.skip_expected(Rule::OF);
            let owner = visit_var(children.consume_expected(Rule::var));
            DeletableKind::Has { attribute, owner }
        }
        Rule::LINKS | Rule::relation => {
            children.try_consume_expected(Rule::LINKS);
            let players = visit_relation(children.consume_expected(Rule::relation));
            children.skip_expected(Rule::OF);
            let relation = visit_var(children.consume_expected(Rule::var));
            DeletableKind::Links { players, relation }
        }
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: children.as_str().to_owned() }),
    };
    debug_assert_eq!(children.try_consume_any(), None);
    Deletable::new(span, kind)
}

fn visit_pattern_try_deletable(node: Node<'_>) -> Deletable {
    debug_assert_eq!(node.as_rule(), Rule::pattern_try_deletable);
    let span = node.span();
    let deletables = node
        .into_children()
        .skip_expected(Rule::TRY)
        .map(|child| match child.as_rule() {
            Rule::statement_deletable => visit_statement_deletable(child),
            _ => unreachable!(
                "Unrecognised statement inside delete clause: {:?}",
                TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }
            ),
        })
        .collect();
    Deletable::new(span, DeletableKind::Optional { deletables })
}

fn visit_pattern_if_deletable(node: Node<'_>) -> Deletable {
    debug_assert_eq!(node.as_rule(), Rule::pattern_if_deletable);
    let span = node.span();
    let mut children = node.into_children();
    let conditions = visit_write_if_clause(children.consume_expected(Rule::write_if_clause));
    let deletables = children
        .map(|child| match child.as_rule() {
            Rule::statement_deletable => visit_statement_deletable(child),
            _ => unreachable!(
                "Unrecognised statement inside if-deletable: {:?}",
                TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }
            ),
        })
        .collect();
    Deletable::new(span, DeletableKind::If { conditions, deletables })
}

fn visit_write_if_clause(node: Node<'_>) -> Vec<WriteCondition> {
    debug_assert_eq!(node.as_rule(), Rule::write_if_clause);
    node.into_children()
        .skip_expected(Rule::IF)
        .map(|child| match child.as_rule() {
            Rule::write_condition => visit_write_condition(child),
            _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
        })
        .collect()
}

fn visit_write_condition(node: Node<'_>) -> WriteCondition {
    debug_assert_eq!(node.as_rule(), Rule::write_condition);
    let mut children = node.into_children();
    match children.peek_rule().unwrap() {
        Rule::statement_isset => {
            WriteCondition::IsSet(visit_statement_isset(children.consume_expected(Rule::statement_isset)))
        }
        Rule::statement_comparison => WriteCondition::Comparison(visit_statement_comparison(
            children.consume_expected(Rule::statement_comparison),
        )),
        Rule::var => {
            let variable = visit_var(children.consume_expected(Rule::var));
            let isa = visit_isa_constraint(children.consume_expected(Rule::isa_constraint));
            WriteCondition::Isa { variable, isa }
        }
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: children.as_str().to_owned() }),
    }
}

fn visit_clause_fetch(node: Node<'_>) -> Fetch {
    debug_assert_eq!(node.as_rule(), Rule::clause_fetch);
    let span = node.span();
    let mut children = node.into_children();
    let fetch_object = visit_fetch_object(children.skip_expected(Rule::FETCH).consume_expected(Rule::fetch_object));
    debug_assert_eq!(children.try_consume_any(), None);
    Fetch::new(span, fetch_object)
}

fn visit_fetch_some(node: Node<'_>) -> FetchSome {
    debug_assert_eq!(node.as_rule(), Rule::fetch_some);
    let child = node.into_child();
    match child.as_rule() {
        Rule::fetch_object => FetchSome::Object(visit_fetch_object(child)),
        Rule::fetch_list => FetchSome::List(visit_fetch_list(child)),
        Rule::fetch_single => FetchSome::Single(visit_fetch_single(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_fetch_single(node: Node<'_>) -> FetchSingle {
    debug_assert_eq!(node.as_rule(), Rule::fetch_single);
    let child = node.into_child();
    match child.as_rule() {
        Rule::fetch_attribute => FetchSingle::Attribute(visit_fetch_attribute(child)),
        Rule::function_block => FetchSingle::FunctionBlock(visit_function_block(child)),
        Rule::expression => FetchSingle::Expression(visit_expression(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_fetch_attribute(node: Node<'_>) -> FetchAttribute {
    debug_assert_eq!(node.as_rule(), Rule::fetch_attribute);
    let span = node.span();
    let mut children = node.into_children();
    let owner = visit_var_or_optional(children.consume_expected(Rule::var_or_optional));
    let child = children.consume_any();
    let attribute = match child.as_rule() {
        Rule::label_list => TypeRefAny::List(visit_label_list(child)),
        Rule::label => TypeRefAny::Type(TypeRef::Label(visit_label(child))),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    };
    debug_assert_eq!(children.try_consume_any(), None);
    FetchAttribute::new(span, owner, attribute)
}

fn visit_fetch_object(node: Node<'_>) -> FetchObject {
    debug_assert_eq!(node.as_rule(), Rule::fetch_object);
    let span = node.span();
    let mut children = node.into_children();
    let body = visit_fetch_object_body(children.consume_expected(Rule::fetch_body));
    FetchObject::new(span, body)
}

fn visit_fetch_object_body(node: Node<'_>) -> FetchObjectBody {
    debug_assert_eq!(node.as_rule(), Rule::fetch_body);
    let child = node.into_child();
    match child.as_rule() {
        Rule::fetch_object_entries => {
            let entries = child.into_children().map(visit_fetch_object_entry).collect();
            FetchObjectBody::Entries(entries)
        }
        Rule::fetch_attributes_all => {
            let var = visit_var_or_optional(child.into_children().consume_expected(Rule::var_or_optional));
            FetchObjectBody::AttributesAll(var)
        }
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_fetch_object_entry(node: Node<'_>) -> FetchObjectEntry {
    debug_assert_eq!(node.as_rule(), Rule::fetch_object_entry);
    let span = node.span();
    let mut children = node.into_children();
    let key = visit_fetch_key(children.consume_expected(Rule::fetch_key));
    let value = visit_fetch_some(children.consume_expected(Rule::fetch_some));
    debug_assert_eq!(children.try_consume_any(), None);
    FetchObjectEntry::new(span, key, value)
}

fn visit_fetch_key(node: Node<'_>) -> StringLiteral {
    debug_assert_eq!(node.as_rule(), Rule::fetch_key);
    visit_quoted_string_literal(node.into_child())
}

fn visit_fetch_list(node: Node<'_>) -> FetchList {
    debug_assert_eq!(node.as_rule(), Rule::fetch_list);
    let span = node.span();
    let stream = visit_fetch_stream(node.into_child());
    FetchList::new(span, stream)
}

fn visit_fetch_stream(node: Node<'_>) -> FetchStream {
    debug_assert_eq!(node.as_rule(), Rule::fetch_stream);
    let child = node.into_child();
    match child.as_rule() {
        Rule::fetch_attribute => FetchStream::Attribute(visit_fetch_attribute(child)),
        Rule::function_block => FetchStream::SubQueryFunctionBlock(visit_function_block(child)),
        Rule::query_pipeline => FetchStream::SubQueryFetch(visit_query_pipeline(child)),
        Rule::expression_function => FetchStream::Function(visit_expression_function(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

pub(super) fn visit_operator_stream(node: Node<'_>) -> Operator {
    debug_assert_eq!(node.as_rule(), Rule::operator_stream);
    let child = node.into_child();
    match child.as_rule() {
        Rule::operator_select => Operator::Select(visit_operator_select(child)),
        Rule::operator_sort => Operator::Sort(visit_operator_sort(child)),
        Rule::operator_offset => Operator::Offset(visit_operator_offset(child)),
        Rule::operator_limit => Operator::Limit(visit_operator_limit(child)),
        Rule::operator_reduce => Operator::Reduce(visit_operator_reduce(child)),
        Rule::operator_require => Operator::Require(visit_operator_require(child)),
        Rule::operator_distinct => Operator::Distinct(visit_operator_distinct(child)),
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_operator_reduce(node: Node<'_>) -> Reduce {
    debug_assert_eq!(node.as_rule(), Rule::operator_reduce);
    let span = node.span();
    let mut children = node.into_children();
    let mut reduce_assignments = Vec::new();
    let mut group = None;
    children.skip_expected(Rule::REDUCE);
    while let Some(child) = children.try_consume_any() {
        match child.as_rule() {
            Rule::reduce_assign => reduce_assignments.push(visit_reduce_assign(child)),
            Rule::GROUPBY => {
                debug_assert!(!reduce_assignments.is_empty());
                group = Some(visit_vars(children.consume_expected(Rule::vars)));
                break;
            }
            _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
        }
    }
    debug_assert_eq!(children.try_consume_any(), None);
    Reduce::new(span, reduce_assignments, group)
}

pub(super) fn visit_reduce_assign(node: Node<'_>) -> ReduceAssign {
    debug_assert_eq!(node.as_rule(), Rule::reduce_assign);
    let mut children = node.into_children();
    let variable = visit_var_or_optional(children.consume_expected(Rule::var_or_optional));
    children.consume_expected(Rule::ASSIGN);
    let reducer = visit_reducer(children.consume_expected(Rule::reducer));
    ReduceAssign { variable, reducer }
}

pub(crate) fn visit_reducer(node: Node<'_>) -> Reducer {
    debug_assert_eq!(node.as_rule(), Rule::reducer);
    let span = node.span();
    let mut children = node.into_children();
    let keyword = children.consume_any();
    match keyword.as_rule() {
        Rule::COUNT => Reducer::Count(Count::new(span, children.try_consume_expected(Rule::var).map(visit_var))),
        Rule::reducer_stat => {
            let operator = visit_reducer_stat(keyword);
            Reducer::Stat(Stat::new(span, operator, visit_var(children.consume_expected(Rule::var))))
        }
        Rule::reducer_collect => {
            let operator = visit_reducer_collect(keyword);
            Reducer::Collect(Collect::new(span, operator, visit_var(children.consume_expected(Rule::var))))
        }
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: keyword.as_str().to_owned() }),
    }
}

fn visit_reducer_stat(node: Node<'_>) -> ReduceOperatorStat {
    debug_assert_eq!(node.as_rule(), Rule::reducer_stat);
    let child = node.into_child();
    match child.as_rule() {
        Rule::MAX => ReduceOperatorStat::Max,
        Rule::MIN => ReduceOperatorStat::Min,
        Rule::MEAN => ReduceOperatorStat::Mean,
        Rule::MEDIAN => ReduceOperatorStat::Median,
        Rule::STD => ReduceOperatorStat::Std,
        Rule::SUM => ReduceOperatorStat::Sum,
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_reducer_collect(node: Node<'_>) -> ReduceOperatorCollect {
    debug_assert_eq!(node.as_rule(), Rule::reducer_collect);
    let child = node.into_child();
    match child.as_rule() {
        Rule::LIST => ReduceOperatorCollect::List,
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_operator_select(node: Node<'_>) -> Select {
    debug_assert_eq!(node.as_rule(), Rule::operator_select);
    let span = node.span();
    let mut children = node.into_children();
    let variables = visit_vars(children.skip_expected(Rule::SELECT).consume_expected(Rule::vars));
    debug_assert_eq!(children.try_consume_any(), None);
    Select::new(span, variables)
}

fn visit_operator_sort(node: Node<'_>) -> Sort {
    debug_assert_eq!(node.as_rule(), Rule::operator_sort);
    let span = node.span();
    let ordered_variables = node.into_children().skip_expected(Rule::SORT).map(visit_var_order).collect();
    Sort::new(span, ordered_variables)
}

fn visit_var_order(node: Node<'_>) -> OrderedVariable {
    debug_assert_eq!(node.as_rule(), Rule::var_order);
    let span = node.span();
    let mut children = node.into_children();
    let variable = visit_var(children.consume_expected(Rule::var));
    let ordering = children.try_consume_expected(Rule::ORDER).map(visit_order);
    debug_assert_eq!(children.try_consume_any(), None);
    OrderedVariable::new(span, variable, ordering)
}

fn visit_order(node: Node<'_>) -> Order {
    debug_assert_eq!(node.as_rule(), Rule::ORDER);
    let child = node.into_child();
    match child.as_rule() {
        Rule::ASC => Order::Asc,
        Rule::DESC => Order::Desc,
        _ => unreachable!("{}", TypeQLError::IllegalGrammar { input: child.as_str().to_owned() }),
    }
}

fn visit_operator_offset(node: Node<'_>) -> Offset {
    debug_assert_eq!(node.as_rule(), Rule::operator_offset);
    let span = node.span();
    let mut children = node.into_children();
    let offset = visit_integer_literal(children.skip_expected(Rule::OFFSET).consume_expected(Rule::integer_literal));
    debug_assert_eq!(children.try_consume_any(), None);
    Offset::new(span, offset)
}

fn visit_operator_limit(node: Node<'_>) -> Limit {
    debug_assert_eq!(node.as_rule(), Rule::operator_limit);
    let span = node.span();
    let mut children = node.into_children();
    let limit = visit_integer_literal(children.skip_expected(Rule::LIMIT).consume_expected(Rule::integer_literal));
    debug_assert_eq!(children.try_consume_any(), None);
    Limit::new(span, limit)
}

fn visit_operator_require(node: Node<'_>) -> Require {
    debug_assert_eq!(node.as_rule(), Rule::operator_require);
    let span = node.span();
    let mut children = node.into_children();
    let variables = visit_vars(children.skip_expected(Rule::REQUIRE).consume_expected(Rule::vars));
    debug_assert_eq!(children.try_consume_any(), None);
    Require::new(span, variables)
}

fn visit_operator_distinct(node: Node<'_>) -> Distinct {
    debug_assert_eq!(node.as_rule(), Rule::operator_distinct);
    let span = node.span();
    let mut children = node.into_children();
    let _ = children.skip_expected(Rule::DISTINCT);
    debug_assert_eq!(children.try_consume_any(), None);
    Distinct::new(span)
}
