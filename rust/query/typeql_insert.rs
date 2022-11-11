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
        error::{collect_err, ErrorMessage, NO_VARIABLE_IN_SCOPE_INSERT},
        token,
        validatable::Validatable,
    },
    pattern::ThingVariable,
    query::{writable::expect_non_empty, TypeQLMatch},
    write_joined,
};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeQLInsert {
    pub match_query: Option<TypeQLMatch>,
    pub variables: Vec<ThingVariable>,
}

impl TypeQLInsert {
    pub fn new(variables: Vec<ThingVariable>) -> Self {
        TypeQLInsert { match_query: None, variables }
    }
}

impl Validatable for TypeQLInsert {
    fn validate(&self) -> Result<(), Vec<ErrorMessage>> {
        collect_err(
            &mut [
                expect_non_empty(&self.variables),
                expect_insert_in_scope_of_match(&self.match_query, &self.variables),
            ]
            .into_iter()
            .chain(self.match_query.iter().map(Validatable::validate))
            .chain(self.variables.iter().map(Validatable::validate)),
        )
    }
}

fn expect_insert_in_scope_of_match(
    match_query: &Option<TypeQLMatch>,
    variables: &Vec<ThingVariable>,
) -> Result<(), Vec<ErrorMessage>> {
    if let Some(match_query) = match_query {
        let bounds = match_query.conjunction.names();
        if variables.iter().any(|v| {
            v.reference.is_name() && bounds.contains(&v.reference.to_string())
                || v.references().any(|w| bounds.contains(&w.to_string()))
        }) {
            Ok(())
        } else {
            let variables_str =
                variables.iter().map(ThingVariable::to_string).collect::<Vec<String>>().join(", ");
            let bounds_str = bounds.into_iter().collect::<Vec<String>>().join(", ");
            Err(vec![NO_VARIABLE_IN_SCOPE_INSERT.format(&[&variables_str, &bounds_str])])
        }
    } else {
        Ok(())
    }
}

impl fmt::Display for TypeQLInsert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(match_query) = &self.match_query {
            writeln!(f, "{}", match_query)?;
        }

        writeln!(f, "{}", token::Command::Insert)?;
        write_joined!(f, ";\n", self.variables)?;
        f.write_str(";")
    }
}
