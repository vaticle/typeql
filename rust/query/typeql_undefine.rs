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
        token,
    },
    pattern::{Definable, RuleDeclaration, TypeVariable},
    write_joined,
};
use std::fmt;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct TypeQLUndefine {
    variables: Vec<TypeVariable>,
    rules: Vec<RuleDeclaration>,
}

impl TypeQLUndefine {
    pub fn new(undefinables: Vec<Definable>) -> Self {
        undefinables.into_iter().fold(TypeQLUndefine::default(), |undefine, undefinable| {
            match undefinable {
                Definable::RuleDeclaration(rule) => undefine.add_rule(rule),
                Definable::TypeVariable(var) => undefine.add_definition(var),
                Definable::RuleDefinition(_r) => {
                    unreachable!() // TODO validation
                                   // Err(INVALID_RULE_WHEN_MISSING_PATTERNS.format(&[&r.to_string()]))
                }
            }
        })
    }

    fn add_definition(mut self, variable: TypeVariable) -> Self {
        self.variables.push(variable);
        self
    }

    fn add_rule(mut self, rule: RuleDeclaration) -> Self {
        self.rules.push(rule);
        self
    }
}

impl fmt::Display for TypeQLUndefine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", token::Command::Undefine)?;
        write_joined!(f, ";\n", self.variables)?;
        write_joined!(f, ";\n", self.rules)?;
        f.write_str(";")
    }
}
