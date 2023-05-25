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
 */

package com.vaticle.typeql.lang.pattern.constraint;

import com.vaticle.typedb.common.collection.Either;
import com.vaticle.typedb.common.collection.Pair;
import com.vaticle.typeql.lang.common.TypeQLArg;
import com.vaticle.typeql.lang.common.TypeQLToken;
import com.vaticle.typeql.lang.common.TypeQLToken.Annotation;
import com.vaticle.typeql.lang.common.exception.TypeQLException;
import com.vaticle.typeql.lang.pattern.variable.TypeVariable;
import com.vaticle.typeql.lang.pattern.variable.UnboundVariable;

import javax.annotation.Nullable;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.regex.PatternSyntaxException;

import static com.vaticle.typedb.common.collection.Collections.set;
import static com.vaticle.typedb.common.util.Objects.className;
import static com.vaticle.typeql.lang.common.TypeQLToken.Char.COLON;
import static com.vaticle.typeql.lang.common.TypeQLToken.Char.SPACE;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.AS;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.OWNS;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.PLAYS;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.REGEX;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.RELATES;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.SUB;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.SUBX;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.TYPE;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.VALUE_TYPE;
import static com.vaticle.typeql.lang.common.TypeQLToken.Type.RELATION;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.INVALID_ANNOTATION;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.INVALID_ATTRIBUTE_TYPE_REGEX;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.INVALID_CASTING;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.MISSING_PATTERNS;
import static com.vaticle.typeql.lang.common.util.Strings.escapeRegex;
import static com.vaticle.typeql.lang.common.util.Strings.escapeString;
import static com.vaticle.typeql.lang.common.util.Strings.quoteString;
import static com.vaticle.typeql.lang.pattern.variable.UnboundVariable.hidden;

public abstract class TypeConstraint extends Constraint<TypeVariable> {

    @Override
    public Set<TypeVariable> variables() {
        return set();
    }

    @Override
    public boolean isType() {
        return true;
    }

    @Override
    public TypeConstraint asType() {
        return this;
    }

    public boolean isLabel() {
        return false;
    }

    public boolean isSub() {
        return false;
    }

    public boolean isAbstract() {
        return false;
    }

    public boolean isValueType() {
        return false;
    }

    public boolean isRegex() {
        return false;
    }

    public boolean isOwns() {
        return false;
    }

    public boolean isPlays() {
        return false;
    }

    public boolean isRelates() {
        return false;
    }

    public List<Annotation> annotations() {
        return Collections.emptyList();
    }

    public TypeConstraint.Label asLabel() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Label.class)));
    }

    public TypeConstraint.Sub asSub() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Sub.class)));
    }

    public TypeConstraint.Abstract asAbstract() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Abstract.class)));
    }

    public TypeConstraint.ValueType asValueType() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(ValueType.class)));
    }

    public TypeConstraint.Regex asRegex() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Regex.class)));
    }

    public TypeConstraint.Owns asOwns() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Owns.class)));
    }

    public TypeConstraint.Plays asPlays() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Plays.class)));
    }

    public TypeConstraint.Relates asRelates() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Relates.class)));
    }

    public static class Label extends TypeConstraint {

        private final String label;
        private final String scope;
        private final int hash;

        public Label(String label) {
            this(null, label);
        }

        public Label(@Nullable String scope, String label) {
            if (label == null) throw new NullPointerException("Null label");
            this.scope = scope;
            this.label = label;
            this.hash = Objects.hash(Label.class, this.scope, this.label);
        }

        public Optional<String> scope() {
            return Optional.ofNullable(scope);
        }

        public String label() {
            return label;
        }

        public String scopedLabel() {
            if (scope != null) return scope + COLON + label;
            else return label;
        }

        @Override
        public boolean isLabel() {
            return true;
        }

        @Override
        public TypeConstraint.Label asLabel() {
            return this;
        }

        @Override
        public String toString() {
            return TYPE.toString() + SPACE + scopedLabel();
        }

        @Override
        public boolean equals(Object o) {
            if (o == this) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Label that = (Label) o;
            return (Objects.equals(this.scope, that.scope) && this.label.equals(that.label));
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public static class Sub extends TypeConstraint {

        private final TypeVariable type;
        private final boolean isExplicit;
        private final int hash;

        public Sub(String typeLabel, boolean isExplicit) {
            this(hidden().type(typeLabel), isExplicit);
        }

        public Sub(String typeScope, String typeLabel, boolean isExplicit) {
            this(hidden().type(typeScope, typeLabel), isExplicit);
        }

        public Sub(UnboundVariable typeVar, boolean isExplicit) {
            this(typeVar.toType(), isExplicit);
        }

        public Sub(Either<Pair<String, String>, UnboundVariable> typeArg, boolean isExplicit) {
            this(typeArg.apply(scoped -> hidden().constrain(new TypeConstraint.Label(scoped.first(), scoped.second())),
                    UnboundVariable::toType), isExplicit);
        }

        private Sub(TypeVariable type, boolean isExplicit) {
            if (type == null) throw new NullPointerException("Null superType");
            this.type = type;
            this.isExplicit = isExplicit;
            this.hash = Objects.hash(Sub.class, this.type, this.isExplicit);
        }

        public TypeVariable type() {
            return type;
        }

        public boolean isExplicit() {
            return isExplicit;
        }

        @Override
        public Set<TypeVariable> variables() {
            return set(type);
        }

        @Override
        public boolean isSub() {
            return true;
        }

        @Override
        public TypeConstraint.Sub asSub() {
            return this;
        }

        @Override
        public String toString() {
            return (isExplicit ? SUBX.toString() : SUB.toString()) + SPACE + type();
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Sub that = (Sub) o;
            return (this.type.equals(that.type) && this.isExplicit == that.isExplicit);
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public static class Abstract extends TypeConstraint {

        private final int hash;

        public Abstract() {
            this.hash = Objects.hash(Abstract.class);
        }

        @Override
        public boolean isAbstract() {
            return true;
        }

        @Override
        public TypeConstraint.Abstract asAbstract() {
            return this;
        }

        @Override
        public String toString() {
            return TypeQLToken.Constraint.ABSTRACT.toString();
        }

        @Override
        public boolean equals(Object o) {
            if (o == this) return true;
            return o != null && getClass() == o.getClass();
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public static class ValueType extends TypeConstraint {

        private final TypeQLArg.ValueType valueType;
        private final int hash;

        public ValueType(TypeQLArg.ValueType valueType) {
            if (valueType == null) throw new NullPointerException("Null ValueType");
            this.valueType = valueType;
            this.hash = Objects.hash(ValueType.class, this.valueType);
        }

        public TypeQLArg.ValueType valueType() {
            return valueType;
        }

        @Override
        public boolean isValueType() {
            return true;
        }

        @Override
        public TypeConstraint.ValueType asValueType() {
            return this;
        }

        @Override
        public String toString() {
            return VALUE_TYPE.toString() + SPACE + valueType.toString();
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            ValueType that = (ValueType) o;
            return (this.valueType.equals(that.valueType));
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public static class Regex extends TypeConstraint {

        private final java.util.regex.Pattern regex;
        private final int hash;

        public Regex(String regex) {
            if (regex == null) throw new NullPointerException("Null regex");
            try {
                this.regex = java.util.regex.Pattern.compile(regex);
            } catch (PatternSyntaxException exception) {
                throw TypeQLException.of(INVALID_ATTRIBUTE_TYPE_REGEX.message());
            }
            this.hash = Objects.hash(Regex.class, this.regex.pattern());
        }

        public java.util.regex.Pattern regex() {
            return regex;
        }

        @Override
        public boolean isRegex() {
            return true;
        }

        @Override
        public TypeConstraint.Regex asRegex() {
            return this;
        }

        @Override
        public String toString() {
            return REGEX.toString() + SPACE + quoteString(escapeString(escapeRegex(regex().pattern())));
        }

        @Override
        public boolean equals(Object o) {
            if (o == this) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Regex that = (Regex) o;
            return (this.regex.pattern().equals(that.regex.pattern()));
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public static class Owns extends TypeConstraint {

        private static final Set<Annotation> VALID_ANNOTATIONS = set(Annotation.KEY, Annotation.UNIQUE);

        private final TypeVariable attributeType;
        private final TypeVariable overriddenAttributeType;
        private final List<Annotation> annotations;
        private final int hash;

        public Owns(String attributeType, Annotation... annotations) {
            this(hidden().type(attributeType), null, annotations);
        }

        public Owns(UnboundVariable attributeTypeVar, Annotation... annotations) {
            this(attributeTypeVar.toType(), null, annotations);
        }

        public Owns(String attributeType, String overriddenAttributeType, Annotation... annotations) {
            this(hidden().type(attributeType), overriddenAttributeType == null ? null : hidden().type(overriddenAttributeType), annotations);
        }

        public Owns(UnboundVariable attributeTypeVar, String overriddenAttributeType, Annotation... annotations) {
            this(attributeTypeVar.toType(), overriddenAttributeType == null ? null : hidden().type(overriddenAttributeType), annotations);
        }

        public Owns(String attributeType, UnboundVariable overriddenAttributeTypeVar, Annotation... annotations) {
            this(hidden().type(attributeType), overriddenAttributeTypeVar == null ? null : overriddenAttributeTypeVar.toType(), annotations);
        }

        public Owns(UnboundVariable attributeTypeVar, UnboundVariable overriddenAttributeTypeVar, Annotation... annotations) {
            this(attributeTypeVar.toType(), overriddenAttributeTypeVar == null ? null : overriddenAttributeTypeVar.toType(), annotations);
        }

        public Owns(Either<String, UnboundVariable> attributeTypeArg, Either<String, UnboundVariable> overriddenAttributeTypeArg, Annotation... annotations) {
            this(attributeTypeArg.apply(label -> hidden().type(label), UnboundVariable::toType),
                    overriddenAttributeTypeArg == null ? null : overriddenAttributeTypeArg.apply(label -> hidden().type(label), UnboundVariable::toType),
                    annotations);
        }

        private Owns(TypeVariable attributeType, @Nullable TypeVariable overriddenAttributeType, Annotation... annotations) {
            this.attributeType = attributeType;
            this.overriddenAttributeType = overriddenAttributeType;
            validateAnnotations(annotations);
            this.annotations = List.of(annotations);
            this.hash = Objects.hash(Owns.class, this.attributeType, this.overriddenAttributeType, this.annotations);
        }

        private static void validateAnnotations(Annotation[] annotations) {
            for (Annotation annotation : annotations) {
                if (!VALID_ANNOTATIONS.contains(annotation)) {
                    throw TypeQLException.of(INVALID_ANNOTATION.message(annotation, "owns"));
                }
            }
        }

        public TypeVariable attribute() {
            return attributeType;
        }

        public Optional<TypeVariable> overridden() {
            return Optional.ofNullable(overriddenAttributeType);
        }

        @Override
        public Set<TypeVariable> variables() {
            return overriddenAttributeType == null
                    ? set(attributeType)
                    : set(attributeType, overriddenAttributeType);
        }

        @Override
        public boolean isOwns() {
            return true;
        }

        public TypeConstraint.Owns asOwns() {
            return this;
        }

        @Override
        public List<Annotation> annotations() {
            return annotations;
        }

        @Override
        public String toString() {
            return "" + OWNS + SPACE + attributeType +
                    (overriddenAttributeType != null ? "" + SPACE + AS + SPACE + overriddenAttributeType : "") +
                    (!annotations.isEmpty() ? SPACE + annotations.stream().map(Annotation::toString).collect(SPACE.joiner()) : "");
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Owns that = (Owns) o;
            return (this.attributeType.equals(that.attributeType) &&
                    Objects.equals(this.overriddenAttributeType, that.overriddenAttributeType) &&
                    this.annotations.equals(that.annotations));
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public static class Plays extends TypeConstraint {

        private final TypeVariable roleType;
        private final TypeVariable relationType;
        private final TypeVariable overriddenRoleType;
        private final int hash;

        public Plays(String relationType, String roleType) {
            this(hidden().type(relationType, roleType), null);
        }

        public Plays(UnboundVariable var) {
            this(var.toType(), null);
        }

        public Plays(String relationType, String roleType, String overriddenRoleType) {
            this(hidden().type(relationType, roleType), overriddenRoleType == null ? null : scopedType(overriddenRoleType));
        }

        public Plays(UnboundVariable roleTypeVar, String overriddenRoleType) {
            this(roleTypeVar.toType(), overriddenRoleType == null ? null : scopedType(overriddenRoleType));
        }

        public Plays(String relationType, String roleType, UnboundVariable overriddenRoleTypeVar) {
            this(hidden().type(relationType, roleType), overriddenRoleTypeVar == null ? null : overriddenRoleTypeVar.toType());
        }

        public Plays(UnboundVariable roleTypeVar, UnboundVariable overriddenRoleTypeVar) {
            this(roleTypeVar.toType(), overriddenRoleTypeVar == null ? null : overriddenRoleTypeVar.toType());
        }

        public Plays(Either<Pair<String, String>, UnboundVariable> roleTypeArg, Either<String, UnboundVariable> overriddenRoleTypeArg) {
            this(roleTypeArg.apply(scoped -> hidden().constrain(new TypeConstraint.Label(scoped.first(), scoped.second())), UnboundVariable::toType),
                    overriddenRoleTypeArg == null ? null : overriddenRoleTypeArg.apply(Plays::scopedType, UnboundVariable::toType));
        }

        private Plays(TypeVariable roleType, @Nullable TypeVariable overriddenRoleType) {
            if (roleType == null) throw TypeQLException.of(MISSING_PATTERNS.message());
            this.relationType = roleType.label().map(l -> hidden().type(l.scope().get())).orElse(null);
            this.roleType = roleType;
            this.overriddenRoleType = overriddenRoleType;
            this.hash = Objects.hash(Plays.class, this.relationType, this.roleType, this.overriddenRoleType);
        }

        public Optional<TypeVariable> relation() {
            return Optional.ofNullable(relationType);
        }

        public TypeVariable role() {
            return roleType;
        }

        public Optional<TypeVariable> overridden() {
            return Optional.ofNullable(overriddenRoleType);
        }

        private static TypeVariable scopedType(String roleType) {
            return hidden().type(RELATION.toString(), roleType);
        }

        @Override
        public Set<TypeVariable> variables() {
            Set<TypeVariable> variables = new HashSet<>();
            variables.add(roleType);
            if (relationType != null) variables.add(relationType);
            if (overriddenRoleType != null) variables.add(overriddenRoleType);
            return variables;
        }

        @Override
        public boolean isPlays() {
            return true;
        }

        @Override
        public TypeConstraint.Plays asPlays() {
            return this;
        }

        @Override
        public String toString() {
            String syntax = PLAYS.toString() + SPACE + roleType;
            if (overriddenRoleType != null) {
                String overriddenRoleTypeString = overriddenRoleType.label().isPresent() ?
                        overriddenRoleType.label().get().label() :
                        overriddenRoleType.reference().syntax();
                syntax += "" + SPACE + AS + SPACE + overriddenRoleTypeString;
            }
            return syntax;
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Plays that = (Plays) o;
            return (this.roleType.equals(that.roleType) &&
                    Objects.equals(this.relationType, that.relationType) &&
                    Objects.equals(this.overriddenRoleType, that.overriddenRoleType));
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public static class Relates extends TypeConstraint {

        private TypeVariable roleType;
        private TypeVariable overriddenRoleType;

        public Relates(String roleType) {
            this(scopedType(roleType), null);
        }

        public Relates(UnboundVariable roleTypeVar) {
            this(roleTypeVar.toType(), null);
        }

        public Relates(String roleType, String overriddenRoleType) {
            this(scopedType(roleType), overriddenRoleType == null ? null : scopedType(overriddenRoleType));
        }

        public Relates(UnboundVariable roleTypeVar, String overriddenRoleType) {
            this(roleTypeVar.toType(), overriddenRoleType == null ? null : scopedType(overriddenRoleType));
        }

        public Relates(String roleType, UnboundVariable overriddenRoleTypeVar) {
            this(scopedType(roleType), overriddenRoleTypeVar == null ? null : overriddenRoleTypeVar.toType());
        }

        public Relates(UnboundVariable roleTypeVar, UnboundVariable overriddenRoleTypeVar) {
            this(roleTypeVar.toType(), overriddenRoleTypeVar == null ? null : overriddenRoleTypeVar.toType());
        }

        public Relates(Either<String, UnboundVariable> roleTypeArg, Either<String, UnboundVariable> overriddenRoleTypeArg) {
            this(roleTypeArg.apply(Relates::scopedType, UnboundVariable::toType),
                    overriddenRoleTypeArg == null ? null : overriddenRoleTypeArg.apply(Relates::scopedType, UnboundVariable::toType));
        }

        private Relates(TypeVariable roleType, @Nullable TypeVariable overriddenRoleType) {
            if (roleType == null) throw new NullPointerException("Null role");
            this.roleType = roleType;
            this.overriddenRoleType = overriddenRoleType;
        }

        private static TypeVariable scopedType(String roleType) {
            return hidden().type(RELATION.toString(), roleType);
        }

        public void setScope(String relationLabel) {
            if (roleType.label().isPresent()) {
                this.roleType = hidden().type(relationLabel, roleType.label().get().label());
            }
            if (overriddenRoleType != null && overriddenRoleType.label().isPresent()) {
                this.overriddenRoleType = hidden().type(relationLabel, overriddenRoleType.label().get().label());
            }
        }

        public TypeVariable role() {
            return roleType;
        }

        public Optional<TypeVariable> overridden() {
            return Optional.ofNullable(overriddenRoleType);
        }

        @Override
        public Set<TypeVariable> variables() {
            return overriddenRoleType == null ? set(roleType) : set(roleType, overriddenRoleType);
        }

        @Override
        public boolean isRelates() {
            return true;
        }

        @Override
        public TypeConstraint.Relates asRelates() {
            return this;
        }

        @Override
        public String toString() {
            StringBuilder syntax = new StringBuilder();
            syntax.append(RELATES).append(SPACE);
            if (!roleType.label().isPresent()) syntax.append(roleType);
            else syntax.append(roleType.label().get().label());
            if (overriddenRoleType != null) {
                syntax.append(SPACE).append(AS).append(SPACE);
                if (!overriddenRoleType.label().isPresent()) syntax.append(overriddenRoleType);
                else syntax.append(overriddenRoleType.label().get().label());
            }
            return syntax.toString();
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Relates that = (Relates) o;
            return (this.roleType.equals(that.roleType) &&
                    Objects.equals(this.overriddenRoleType, that.overriddenRoleType));
        }

        @Override
        public int hashCode() {
            return Objects.hash(Relates.class, roleType, overriddenRoleType);
        }
    }
}
