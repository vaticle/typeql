/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{fmt, iter};

use crate::{
    common::{error::collect_err, validatable::Validatable, Result},
    pattern::{AssignConstraint, Predicate},
    variable::{variable::VariableRef, ValueVariable},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValueStatement {
    pub variable: ValueVariable,
    pub assign_constraint: Option<AssignConstraint>,
    pub predicate_constraint: Option<Predicate>,
}

impl ValueStatement {
    pub fn new(variable: ValueVariable) -> ValueStatement {
        ValueStatement { variable, assign_constraint: None, predicate_constraint: None }
    }

    pub fn owner(&self) -> VariableRef<'_> {
        VariableRef::Value(&self.variable)
    }

    pub fn variables(&self) -> Box<dyn Iterator<Item = VariableRef<'_>> + '_> {
        Box::new(
            iter::once(self.owner())
                .chain(self.assign_constraint.iter().flat_map(|assign| assign.variables()))
                .chain(self.predicate_constraint.iter().flat_map(|predicate| predicate.variables())),
        )
    }

    pub fn constrain_assign(self, assign: AssignConstraint) -> ValueStatement {
        Self { assign_constraint: Some(assign), ..self }
    }

    pub fn constrain_predicate(self, predicate: Predicate) -> ValueStatement {
        Self { predicate_constraint: Some(predicate), ..self }
    }
}

impl Validatable for ValueStatement {
    fn validate(&self) -> Result {
        collect_err(
            iter::once(self.variable.validate())
                .chain(self.assign_constraint.iter().map(Validatable::validate))
                .chain(self.predicate_constraint.iter().map(Validatable::validate)),
        )
    }
}

impl From<ValueVariable> for ValueStatement {
    fn from(variable: ValueVariable) -> Self {
        ValueStatement::new(variable)
    }
}

impl fmt::Display for ValueStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variable)?;
        if let Some(assign) = &self.assign_constraint {
            write!(f, " {assign}")?;
        } else if let Some(predicate) = &self.predicate_constraint {
            write!(f, " {} {}", predicate.predicate, predicate.value)?;
        }
        Ok(())
    }
}
