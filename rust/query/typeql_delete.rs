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

use std::fmt;

use crate::{
    common::{
        error::{collect_err, TypeQLError},
        token,
        validatable::Validatable,
        Result,
    },
    pattern::{NamedReferences, ThingStatement},
    query::{writable::expect_non_empty, TypeQLGet, TypeQLUpdate, Writable},
    write_joined,
};
use crate::query::MatchClause;
use crate::query::modifier::Modifiers;

#[derive(Debug, Eq, PartialEq)]
pub struct TypeQLDelete {
    pub clause_match: MatchClause,
    pub statements: Vec<ThingStatement>,
    pub modifiers: Modifiers,
}

impl TypeQLDelete {
    pub fn insert(self, vars: impl Writable) -> TypeQLUpdate {
        TypeQLUpdate { query_delete: self, insert_statements: vars.vars(), modifiers: Default::default() }
    }
}

impl Validatable for TypeQLDelete {
    fn validate(&self) -> Result<()> {
        collect_err(
            &mut ([
                expect_delete_in_scope_of_match(&self.clause_match, &self.statements),
                expect_non_empty(&self.statements),
                self.clause_match.validate(),
            ]
            .into_iter())
            .chain(self.statements.iter().map(Validatable::validate)),
        )
    }
}

fn expect_delete_in_scope_of_match(clause_match: &MatchClause, variables: &[ThingStatement]) -> Result<()> {
    let names_in_scope = clause_match.named_references();
    collect_err(&mut variables.iter().flat_map(|v| v.references()).filter(|r| r.is_name()).map(|r| -> Result<()> {
        if names_in_scope.contains(r) {
            Ok(())
        } else {
            Err(TypeQLError::VariableOutOfScopeDelete(r.clone()))?
        }
    }))
}

impl fmt::Display for TypeQLDelete {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.clause_match)?;
        writeln!(f, "{}", token::Command::Delete)?;
        write_joined!(f, ";\n", self.statements)?;
        write!(f, "\n{}", self.modifiers)?;
        f.write_str(";")
    }
}
