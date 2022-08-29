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

use std::convert::Into;

use typeql_grammar::typeqlrustlexer::TypeQLRustLexer;
use typeql_grammar::typeqlrustparser::*;
use typeql_grammar::typeqlrustvisitor::TypeQLRustVisitorCompat;

use antlr_rust::common_token_stream::CommonTokenStream;
use antlr_rust::InputStream;

pub mod parser;
pub mod pattern;
pub mod query;

#[macro_use]
mod util;

use parser::Parser;
use pattern::*;
use query::*;

pub fn parse_query(typeql_query: &str) -> Query {
    parse_eof_query(typeql_query)
}

pub fn typeql_match(pattern: impl Into<Conjunction>) -> Query {
    Query::Match(TypeQLMatch {
        conjunction: pattern.into(),
        filter: vec![],
    })
}

pub fn var(name: impl Into<String>) -> UnboundVariable {
    UnboundVariable::named(name.into())
}

pub fn type_(name: impl Into<String>) -> TypeVariable {
    UnboundVariable::hidden().type_(name.into())
}

pub fn rel<T: Into<RolePlayerConstraint>>(value: T) -> ThingVariable {
    UnboundVariable::hidden().rel(value)
}

pub fn parse_eof_query(query_string: &str) -> Query {
    let lexer = TypeQLRustLexer::new(InputStream::new(query_string.into()));
    let mut parser = TypeQLRustParser::new(CommonTokenStream::new(lexer));
    Parser::default()
        .visit_eof_query(parser.eof_query().unwrap().as_ref())
        .into_query()
}
