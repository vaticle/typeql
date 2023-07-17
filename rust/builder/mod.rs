/*
 * Copyright (C) 2022 Vaticle
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 *
 */

use crate::{
    common::token::{Function as FunctionToken, Predicate},
    pattern::{
        Constant, Expression, Function, Negation, PredicateConstraint, RelationVariableBuilder, RolePlayerConstraint,
        RuleDeclaration, ThingVariable, TypeVariable, TypeVariableBuilder, UnboundConceptVariable,
        UnboundValueVariable, UnboundVariable, Value,
    },
    Pattern,
};

#[macro_export]
macro_rules! typeql_match {
    ($($pattern:expr),* $(,)?) => {{
        $crate::query::TypeQLMatch::from_patterns(vec![$($pattern.into()),*])
    }}
}

#[macro_export]
macro_rules! typeql_insert {
    ($($thing_variable:expr),* $(,)?) => {{
        $crate::query::TypeQLInsert::new(vec![$($thing_variable),*])
    }}
}

#[macro_export]
macro_rules! typeql_define {
    ($($pattern:expr),* $(,)?) => {{
        $crate::query::TypeQLDefine::new(vec![$($pattern.into()),*])
    }}
}

#[macro_export]
macro_rules! typeql_undefine {
    ($($pattern:expr),* $(,)?) => {{
        $crate::query::TypeQLUndefine::new(vec![$($pattern.into()),*])
    }}
}

#[macro_export]
macro_rules! and {
    ($($pattern:expr),* $(,)?) => {{
        $crate::pattern::Conjunction::new(vec![$($pattern.into()),*])
    }}
}

#[macro_export]
macro_rules! or {
    ($($pattern:expr),* $(,)?) => {{
        let mut patterns = vec![$($pattern.into()),*];
        match patterns.len() {
            1 => patterns.pop().unwrap(),
            _ => $crate::pattern::Disjunction::new(patterns).into(),
        }
    }}
}

#[macro_export]
macro_rules! max {
    ($($args:expr),* $(,)?) => {{
        max([$($args, )*])
    }}
}

#[macro_export]
macro_rules! min {
    ($($args:expr),* $(,)?) => {{
        min([$($args, )*])
    }}
}

#[macro_export]
macro_rules! filter {
    ($($arg:expr),* $(,)?) => {{
        [$(Into::<UnboundVariable>::into($arg)),*]
    }}
}

#[macro_export]
macro_rules! sort_vars {
    ($($arg:expr),*) => {{
        Sorting::new(vec![$(Into::<OrderedVariable>::into($arg), )*])
    }}
}

pub fn not<T: Into<Pattern>>(pattern: T) -> Negation {
    Negation::new(pattern.into())
}

pub fn rule(name: &str) -> RuleDeclaration {
    RuleDeclaration::from(name)
}

pub fn var(var: impl Into<UnboundVariable>) -> UnboundVariable {
    var.into()
}

pub fn cvar(var: impl Into<UnboundConceptVariable>) -> UnboundConceptVariable {
    var.into()
}

pub fn vvar(var: impl Into<UnboundValueVariable>) -> UnboundValueVariable {
    var.into()
}

pub fn constant(value: impl Into<Value>) -> Expression {
    Expression::Constant(Constant { value: value.into() })
}

pub fn type_(name: impl Into<String>) -> TypeVariable {
    UnboundConceptVariable::hidden().type_(name.into())
}

pub fn rel<T: Into<RolePlayerConstraint>>(value: T) -> ThingVariable {
    UnboundConceptVariable::hidden().rel(value)
}

pub fn eq<T: Into<Value>>(value: T) -> PredicateConstraint {
    PredicateConstraint::new(Predicate::Eq, value.into())
}

pub fn neq<T: Into<Value>>(value: T) -> PredicateConstraint {
    PredicateConstraint::new(Predicate::Neq, value.into())
}

pub fn lt<T: Into<Value>>(value: T) -> PredicateConstraint {
    PredicateConstraint::new(Predicate::Lt, value.into())
}

pub fn lte<T: Into<Value>>(value: T) -> PredicateConstraint {
    PredicateConstraint::new(Predicate::Lte, value.into())
}

pub fn gt<T: Into<Value>>(value: T) -> PredicateConstraint {
    PredicateConstraint::new(Predicate::Gt, value.into())
}

pub fn gte<T: Into<Value>>(value: T) -> PredicateConstraint {
    PredicateConstraint::new(Predicate::Gte, value.into())
}

pub fn contains<T: Into<String>>(value: T) -> PredicateConstraint {
    PredicateConstraint::new(Predicate::Contains, Value::from(value.into()))
}

pub fn like<T: Into<String>>(value: T) -> PredicateConstraint {
    PredicateConstraint::new(Predicate::Like, Value::from(value.into()))
}

pub fn abs<T: Into<Expression>>(arg: T) -> Expression {
    Expression::Function(Function { function_name: FunctionToken::Abs, args: vec![Box::from(arg.into())] })
}

pub fn ceil<T: Into<Expression>>(arg: T) -> Expression {
    Expression::Function(Function { function_name: FunctionToken::Ceil, args: vec![Box::from(arg.into())] })
}

pub fn floor<T: Into<Expression>>(arg: T) -> Expression {
    Expression::Function(Function { function_name: FunctionToken::Floor, args: vec![Box::from(arg.into())] })
}

pub fn max<const N: usize, T: Into<Expression>>(args: [T; N]) -> Expression {
    Expression::Function(Function {
        function_name: FunctionToken::Max,
        args: args.into_iter().map(|arg| Box::new(arg.into())).collect(),
    })
}

pub fn min<const N: usize, T: Into<Expression>>(args: [T; N]) -> Expression {
    Expression::Function(Function {
        function_name: FunctionToken::Min,
        args: args.into_iter().map(|arg| Box::new(arg.into())).collect(),
    })
}

pub fn round<T: Into<Expression>>(arg: T) -> Expression {
    Expression::Function(Function { function_name: FunctionToken::Round, args: vec![Box::from(arg.into())] })
}
