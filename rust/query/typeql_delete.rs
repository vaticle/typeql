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

use crate::common::token::Command::Delete;
use crate::{write_joined, ErrorMessage, Query, ThingVariable, TypeQLMatch};
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub struct TypeQLDelete {
    pub match_query: Option<TypeQLMatch>,
    pub variables: Vec<ThingVariable>,
}

impl TypeQLDelete {
    pub fn new(variables: Vec<ThingVariable>) -> Self {
        TypeQLDelete { match_query: None, variables }
    }

    pub fn into_query(self) -> Query {
        Query::Delete(self)
    }
}

impl fmt::Display for TypeQLDelete {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(match_query) = &self.match_query {
            write!(f, "{}\n", match_query)?;
        }

        write!(f, "{}\n", Delete)?;
        write_joined!(f, ";\n", self.variables)?;
        f.write_str(";")
    }
}

pub trait Deletable {
    fn vars(self) -> Vec<ThingVariable>;
}

impl Deletable for ThingVariable {
    fn vars(self) -> Vec<ThingVariable> {
        vec![self]
    }
}

impl<const N: usize> Deletable for [ThingVariable; N] {
    fn vars(self) -> Vec<ThingVariable> {
        self.to_vec()
    }
}

impl<const N: usize> Deletable for [Result<ThingVariable, ErrorMessage>; N] {
    fn vars(self) -> Vec<ThingVariable> {
        self.into_iter().map(|x| x.unwrap()).collect()
    }
}

impl Deletable for Vec<ThingVariable> {
    fn vars(self) -> Vec<ThingVariable> {
        self
    }
}

impl<U: Deletable> Deletable for Result<U, ErrorMessage> {
    fn vars(self) -> Vec<ThingVariable> {
        self.unwrap().vars()
    }
}

pub trait DeleteQueryBuilder {
    fn delete(self, vars: impl Deletable) -> Result<TypeQLDelete, ErrorMessage>;
}

impl<U: DeleteQueryBuilder> DeleteQueryBuilder for Result<U, ErrorMessage> {
    fn delete(self, vars: impl Deletable) -> Result<TypeQLDelete, ErrorMessage> {
        self?.delete(vars)
    }
}
