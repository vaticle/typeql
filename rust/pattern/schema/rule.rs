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
    common::{
        error::{
            collect_err, ErrorMessage, INVALID_RULE_THEN, INVALID_RULE_THEN_HAS,
            INVALID_RULE_THEN_ROLES, INVALID_RULE_THEN_VARIABLES,
            INVALID_RULE_WHEN_CONTAINS_DISJUNCTION, INVALID_RULE_WHEN_NESTED_NEGATION,
        },
        string::indent,
        token,
        validatable::Validatable,
    },
    pattern::{Conjunction, Pattern, ThingVariable},
    Label,
};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuleDeclaration {
    pub label: Label,
}

impl RuleDeclaration {
    pub fn new(label: Label) -> Self {
        RuleDeclaration { label }
    }

    pub fn when(self, when: Conjunction) -> RuleWhenStub {
        RuleWhenStub { label: self.label, when: when }
    }
}

impl From<&str> for RuleDeclaration {
    fn from(label: &str) -> Self {
        RuleDeclaration::new(Label::from(label))
    }
}

impl fmt::Display for RuleDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", token::Schema::Rule, self.label)
    }
}

pub struct RuleWhenStub {
    pub label: Label,
    pub when: Conjunction,
}

impl RuleWhenStub {
    pub fn then(self, then: ThingVariable) -> RuleDefinition {
        RuleDefinition { label: self.label, when: self.when, then }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuleDefinition {
    pub label: Label,
    pub when: Conjunction,
    pub then: ThingVariable,
}

impl Validatable for RuleDefinition {
    fn validate(&self) -> Result<(), Vec<ErrorMessage>> {
        collect_err(
            &mut [
                expect_only_conjunctions(self.when.patterns.iter(), &self.label),
                expect_infer_single_edge(&self.then, &self.label),
                expect_valid_inference(&self.then, &self.label),
                expect_then_bounded_by_when(&self.then, &self.when, &self.label),
                self.when.validate(),
                self.then.validate(),
            ]
            .into_iter(),
        )
    }
}

fn expect_only_conjunctions<'a>(
    patterns: impl Iterator<Item = &'a Pattern>,
    rule_label: &Label,
) -> Result<(), Vec<ErrorMessage>> {
    collect_err(&mut patterns.map(|p| match p {
        Pattern::Conjunction(c) => expect_only_conjunctions(c.patterns.iter(), rule_label),
        Pattern::Variable(_) => Ok(()),
        Pattern::Disjunction(_) => {
            Err(vec![INVALID_RULE_WHEN_CONTAINS_DISJUNCTION.format(&[&rule_label.to_string()])])
        }
        Pattern::Negation(_) => {
            Err(vec![INVALID_RULE_WHEN_NESTED_NEGATION.format(&[&rule_label.to_string()])])
        }
    }))
}

fn expect_infer_single_edge(
    then: &ThingVariable,
    rule_label: &Label,
) -> Result<(), Vec<ErrorMessage>> {
    if then.has.len() == 1
        && (then.iid.is_none()
            && then.isa.is_none()
            && then.value.is_none()
            && then.relation.is_none())
    {
        Ok(())
    } else if then.relation.is_some()
        && then.isa.is_some()
        && (then.iid.is_none() && then.has.is_empty() && then.value.is_none())
    {
        Ok(())
    } else {
        Err(vec![INVALID_RULE_THEN.format(&[&rule_label.to_string(), &then.to_string()])])
    }
}

fn expect_valid_inference(
    then: &ThingVariable,
    rule_label: &Label,
) -> Result<(), Vec<ErrorMessage>> {
    if let Some(has) = then.has.get(0) {
        if has.type_.is_some() && has.attribute.reference.is_name() {
            Err(vec![INVALID_RULE_THEN_HAS.format(&[
                &rule_label.to_string(),
                &then.to_string(),
                &has.attribute.reference.to_string(),
                &has.type_.as_ref().unwrap().to_string(),
            ])])
        } else {
            Ok(())
        }
    } else if let Some(relation) = &then.relation {
        if relation.role_players.iter().all(|rp| rp.role_type.is_some()) {
            Ok(())
        } else {
            Err(vec![INVALID_RULE_THEN_ROLES.format(&[&rule_label.to_string(), &then.to_string()])])
        }
    } else {
        unreachable!()
    }
}

fn expect_then_bounded_by_when(
    then: &ThingVariable,
    when: &Conjunction,
    rule_label: &Label,
) -> Result<(), Vec<ErrorMessage>> {
    let names = when.names();
    if then.references().filter(|r| r.is_name()).all(|r| names.contains(&r.to_string())) {
        Ok(())
    } else {
        Err(vec![INVALID_RULE_THEN_VARIABLES.format(&[&rule_label.to_string()])])
    }
}

impl fmt::Display for RuleDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}:\n{}",
            token::Schema::Rule,
            self.label,
            indent(&format!(
                "{} {}\n{} {{\n    {};\n}}",
                token::Schema::When,
                self.when,
                token::Schema::Then,
                self.then
            ))
        )
    }
}
