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
import com.vaticle.typeql.lang.common.TypeQLToken;
import com.vaticle.typeql.lang.common.exception.TypeQLException;
import com.vaticle.typeql.lang.common.util.Strings;
import com.vaticle.typeql.lang.pattern.variable.BoundVariable;
import com.vaticle.typeql.lang.pattern.variable.ThingVariable;
import com.vaticle.typeql.lang.pattern.variable.TypeVariable;
import com.vaticle.typeql.lang.pattern.variable.UnboundVariable;

import javax.annotation.Nullable;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.regex.Pattern;

import static com.vaticle.typedb.common.collection.Collections.list;
import static com.vaticle.typedb.common.collection.Collections.pair;
import static com.vaticle.typedb.common.collection.Collections.set;
import static com.vaticle.typedb.common.util.Objects.className;
import static com.vaticle.typeql.lang.common.TypeQLToken.Char.COLON;
import static com.vaticle.typeql.lang.common.TypeQLToken.Char.COMMA_SPACE;
import static com.vaticle.typeql.lang.common.TypeQLToken.Char.PARAN_CLOSE;
import static com.vaticle.typeql.lang.common.TypeQLToken.Char.PARAN_OPEN;
import static com.vaticle.typeql.lang.common.TypeQLToken.Char.SPACE;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.HAS;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.ISA;
import static com.vaticle.typeql.lang.common.TypeQLToken.Constraint.ISAX;
import static com.vaticle.typeql.lang.common.TypeQLToken.Predicate.Equality.EQ;
import static com.vaticle.typeql.lang.common.TypeQLToken.Predicate.SubString.LIKE;
import static com.vaticle.typeql.lang.common.TypeQLToken.Type.RELATION;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.INVALID_CASTING;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.INVALID_CONSTRAINT_DATETIME_PRECISION;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.INVALID_IID_STRING;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.MISSING_CONSTRAINT_PREDICATE;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.MISSING_CONSTRAINT_RELATION_PLAYER;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.MISSING_CONSTRAINT_VALUE;
import static com.vaticle.typeql.lang.common.util.Strings.escapeRegex;
import static com.vaticle.typeql.lang.common.util.Strings.escapeString;
import static com.vaticle.typeql.lang.common.util.Strings.quoteString;
import static com.vaticle.typeql.lang.pattern.variable.UnboundVariable.hidden;

public abstract class ThingConstraint extends Constraint<BoundVariable> {

    @Override
    public Set<BoundVariable> variables() {
        return set();
    }

    @Override
    public boolean isThing() {
        return true;
    }

    @Override
    public ThingConstraint asThing() {
        return this;
    }

    public boolean isIID() {
        return false;
    }

    public boolean isIsa() {
        return false;
    }

    public boolean isValue() {
        return false;
    }

    public boolean isRelation() {
        return false;
    }

    public boolean isHas() {
        return false;
    }

    public IID asIID() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(IID.class)));
    }

    public ThingConstraint.Isa asIsa() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Isa.class)));
    }

    public Value<?> asValue() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Value.class)));
    }

    public ThingConstraint.Relation asRelation() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Relation.class)));
    }

    public ThingConstraint.Has asHas() {
        throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Has.class)));
    }

    public static class IID extends ThingConstraint {

        private static final Pattern REGEX = Pattern.compile("0x[0-9a-f]+");
        private final String iid;
        private final int hash;

        public IID(String iid) {
            if (iid == null) throw new NullPointerException("Null IID");
            if (!REGEX.matcher(iid).matches()) {
                throw TypeQLException.of(INVALID_IID_STRING.message(iid, REGEX.toString()));
            }
            this.iid = iid;
            this.hash = Objects.hash(IID.class, this.iid);
        }

        public String iid() {
            return iid;
        }

        @Override
        public boolean isIID() {
            return true;
        }

        @Override
        public IID asIID() {
            return this;
        }

        @Override
        public String toString() {
            return TypeQLToken.Constraint.IID.toString() + SPACE + iid;
        }

        @Override
        public boolean equals(Object o) {
            if (o == this) return true;
            if (o == null || getClass() != o.getClass()) return false;
            IID that = (IID) o;
            return (this.iid.equals(that.iid));
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public static class Isa extends ThingConstraint {

        private final TypeVariable type;
        private final boolean isExplicit;
        private final boolean isDerived;
        private final int hash;

        public Isa(String type, boolean isExplicit) {
            this(hidden().type(type), isExplicit, false);
        }

        public Isa(UnboundVariable typeVar, boolean isExplicit) {
            this(typeVar.toType(), isExplicit, false);
        }

        public Isa(Either<String, UnboundVariable> typeArg, boolean isExplicit) {
            this(typeArg.apply(label -> hidden().type(label), UnboundVariable::toType), isExplicit, false);
        }

        private Isa(TypeVariable type, boolean isExplicit, boolean isDerived) {
            if (type == null) {
                throw new NullPointerException("Null type");
            }
            this.type = type;
            this.isExplicit = isExplicit;
            this.isDerived = isDerived;
            this.hash = Objects.hash(Isa.class, this.type, this.isExplicit, this.isDerived);
        }

        public TypeVariable type() {
            return type;
        }

        public boolean isExplicit() {
            return isExplicit;
        }

        public boolean isDerived() { return isDerived; }

        @Override
        public Set<BoundVariable> variables() {
            return set(type);
        }

        @Override
        public boolean isIsa() {
            return true;
        }

        @Override
        public ThingConstraint.Isa asIsa() {
            return this;
        }

        @Override
        public String toString() {
            return (isExplicit ? ISAX.toString() : ISA.toString()) + SPACE + type();
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Isa that = (Isa) o;
            return (this.type.equals(that.type) &&
                    this.isExplicit == that.isExplicit &&
                    this.isDerived == that.isDerived);
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public static class Relation extends ThingConstraint {

        private final Map<Pair<TypeVariable, ThingVariable<?>>, AtomicInteger> repetitions;
        private final List<RolePlayer> players;
        private String scope;

        public Relation(RolePlayer player) {
            this(list(player));
        }

        public Relation(List<RolePlayer> players) {
            if (players == null || players.isEmpty()) throw TypeQLException.of(MISSING_CONSTRAINT_RELATION_PLAYER);
            this.repetitions = new HashMap<>();
            this.players = new ArrayList<>();
            this.scope = RELATION.toString();
            registerPlayers(players);
        }

        private void registerPlayers(List<RolePlayer> players) {
            for (RolePlayer player : players) {
                player.setScope(scope);
                player.setRepetition(incrementRepetition(player));
                this.players.add(player);
            }
        }

        private int incrementRepetition(RolePlayer player) {
            return repetitions.computeIfAbsent(pair(player.roleType, player.player),
                                               k -> new AtomicInteger(0)).incrementAndGet();
        }

        public void setScope(String relationLabel) {
            this.scope = relationLabel;
            players.forEach(player -> player.setScope(scope));
        }

        public void addPlayers(RolePlayer player) {
            if (scope != null) player.setScope(scope);
            player.setRepetition(incrementRepetition(player));
            players.add(player);
        }

        public List<RolePlayer> players() {
            return players;
        }

        @Override
        public Set<BoundVariable> variables() {
            Set<BoundVariable> variables = new HashSet<>();
            players().forEach(player -> {
                variables.add(player.player());
                if (player.roleType().isPresent()) variables.add(player.roleType().get());
            });
            return variables;
        }

        @Override
        public boolean isRelation() {
            return true;
        }

        @Override
        public ThingConstraint.Relation asRelation() {
            return this;
        }

        @Override
        public String toString() {
            return PARAN_OPEN + players().stream().map(RolePlayer::toString).collect(COMMA_SPACE.joiner()) + PARAN_CLOSE;
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Relation that = (Relation) o;
            return this.players.equals(that.players);
        }

        @Override
        public int hashCode() {
            return Objects.hash(Relation.class, this.players);
        }

        public static class RolePlayer {

            private TypeVariable roleType;
            private final ThingVariable<?> player;
            private int repetition;

            public RolePlayer(String roleType, UnboundVariable playerVar) {
                this(roleType == null ? null : hidden().type(roleType), playerVar.toThing());
            }

            public RolePlayer(UnboundVariable roleTypeVar, UnboundVariable playerVar) {
                this(roleTypeVar == null ? null : roleTypeVar.toType(), playerVar.toThing());
            }

            public RolePlayer(UnboundVariable playerVar) {
                this(null, playerVar.toThing());
            }

            public RolePlayer(Either<String, UnboundVariable> roleTypeArg, UnboundVariable playerVar) {
                this(roleTypeArg == null ? null : roleTypeArg.apply(label -> hidden().type(label), UnboundVariable::toType), playerVar.toThing());
            }

            private RolePlayer(@Nullable TypeVariable roleType, ThingVariable<?> player) {
                if (player == null) throw new NullPointerException("Null player");
                this.roleType = roleType;
                this.player = player;
            }

            public Optional<TypeVariable> roleType() {
                return Optional.ofNullable(roleType);
            }

            public ThingVariable<?> player() {
                return player;
            }

            public int repetition() {
                return repetition;
            }

            private void setScope(String relationLabel) {
                if (roleType != null && roleType.label().isPresent()) {
                    this.roleType = hidden().type(relationLabel, roleType.label().get().label());
                }
            }

            private void setRepetition(int repetition) {
                this.repetition = repetition;
            }

            @Override
            public String toString() {
                if (roleType == null) {
                    return player.toString();
                } else {
                    StringBuilder syntax = new StringBuilder();
                    if (roleType.isVisible()) syntax.append(roleType.reference().toString());
                    else syntax.append(roleType.label().get().label());
                    syntax.append(COLON).append(SPACE).append(player);
                    return syntax.toString();
                }
            }

            @Override
            public boolean equals(Object o) {
                if (this == o) return true;
                if (o == null || getClass() != o.getClass()) return false;
                RolePlayer that = (RolePlayer) o;
                return (Objects.equals(this.roleType, that.roleType) &&
                        this.player.equals(that.player) &&
                        this.repetition == that.repetition);
            }

            @Override
            public int hashCode() {
                return Objects.hash(RolePlayer.class, roleType, player, repetition);
            }
        }
    }

    public static class Has extends ThingConstraint {

        private final TypeVariable type;
        private final ThingVariable<?> attribute;
        private final int hash;

        public Has(String type, ThingConstraint.Value<?> value) {
            this(hidden().type(type), hidden().constrain(value));
        }

        public Has(String type, UnboundVariable var) {
            this(hidden().type(type), var.toThing());
        }

        public Has(UnboundVariable var) {
            this(null, var.toThing());
        }

        private Has(@Nullable TypeVariable type, ThingVariable<?> attribute) {
            if (attribute == null) throw new NullPointerException("Null attribute");
            this.type = type;
            if (type == null) this.attribute = attribute;
            else this.attribute = attribute.constrain(new Isa(type, false, true));
            this.hash = Objects.hash(Has.class, this.type, this.attribute);
        }

        public ThingVariable<?> attribute() { return attribute; }

        public Optional<TypeVariable> type() { return Optional.ofNullable(type); }

        @Override
        public Set<BoundVariable> variables() {
            return set(attribute);
        }

        @Override
        public boolean isHas() {
            return true;
        }

        @Override
        public ThingConstraint.Has asHas() {
            return this;
        }

        @Override
        public String toString() {
            return String.valueOf(HAS) + SPACE +
                    (type != null ? type.label().get().label() + SPACE : "") +
                    (attribute.isNamed() ? attribute.reference() : attribute.value().get());
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Has that = (Has) o;
            return Objects.equals(this.type, that.type) && this.attribute.equals(that.attribute);
        }

        @Override
        public int hashCode() {
            return hash;
        }
    }

    public abstract static class Value<T> extends ThingConstraint {

        private final TypeQLToken.Predicate predicate;
        private final T value;
        private final int hash;

        Value(TypeQLToken.Predicate predicate, T value) {
            if (predicate == null) throw TypeQLException.of(MISSING_CONSTRAINT_PREDICATE);
            else if (value == null) throw TypeQLException.of(MISSING_CONSTRAINT_VALUE);

            assert !predicate.isEquality() || value instanceof Comparable || value instanceof ThingVariable<?>;
            assert !predicate.isSubString() || value instanceof java.lang.String;

            this.predicate = predicate;
            this.value = value;
            this.hash = Objects.hash(Value.class, this.predicate, this.value);
        }

        @Override
        public Set<BoundVariable> variables() {
            return set();
        }

        @Override
        public boolean isValue() {
            return true;
        }

        @Override
        public Value<?> asValue() {
            return this;
        }

        public TypeQLToken.Predicate predicate() {
            return predicate;
        }

        public T value() {
            return value;
        }

        public boolean isLong() {
            return false;
        }

        public boolean isDouble() {
            return false;
        }

        public boolean isBoolean() {
            return false;
        }

        public boolean isString() {
            return false;
        }

        public boolean isDateTime() {
            return false;
        }

        public boolean isVariable() {
            return false;
        }

        public Long asLong() {
            throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Long.class)));
        }

        public Double asDouble() {
            throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Double.class)));
        }

        public Boolean asBoolean() {
            throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Boolean.class)));
        }

        public String asString() {
            throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(String.class)));
        }

        public DateTime asDateTime() {
            throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(DateTime.class)));
        }

        public Variable asVariable() {
            throw TypeQLException.of(INVALID_CASTING.message(className(this.getClass()), className(Variable.class)));
        }

        @Override
        public java.lang.String toString() {
            if (predicate.equals(EQ) && !isVariable()) return Strings.valueToString(value);
            else return predicate.toString() + SPACE + Strings.valueToString(value);
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Value<?> that = (Value<?>) o;
            return (this.predicate.equals(that.predicate) && this.value.equals(that.value));
        }

        @Override
        public int hashCode() {
            return hash;
        }

        public static class Long extends Value<java.lang.Long> {

            public Long(TypeQLToken.Predicate.Equality predicate, long value) {
                super(predicate, value);
            }

            @Override
            public boolean isLong() {
                return true;
            }

            @Override
            public Long asLong() {
                return this;
            }
        }

        public static class Double extends Value<java.lang.Double> {

            public Double(TypeQLToken.Predicate.Equality predicate, double value) {
                super(predicate, value);
            }

            @Override
            public boolean isDouble() {
                return true;
            }

            @Override
            public Double asDouble() {
                return this;
            }
        }

        public static class Boolean extends Value<java.lang.Boolean> {

            public Boolean(TypeQLToken.Predicate.Equality predicate, boolean value) {
                super(predicate, value);
            }

            @Override
            public boolean isBoolean() {
                return true;
            }

            @Override
            public Boolean asBoolean() {
                return this;
            }
        }

        public static class String extends Value<java.lang.String> {

            public String(TypeQLToken.Predicate predicate, java.lang.String value) {
                super(predicate, value);
            }

            @Override
            public boolean isString() {
                return true;
            }

            @Override
            public java.lang.String toString() {
                StringBuilder operation = new StringBuilder();

                if (predicate().equals(LIKE)) {
                    operation.append(LIKE).append(SPACE).append(quoteString(escapeString(escapeRegex(value()))));
                } else if (predicate().equals(EQ)) {
                    operation.append(quoteString(escapeString(value())));
                } else {
                    operation.append(predicate()).append(SPACE).append(quoteString(escapeString(value())));
                }

                return operation.toString();
            }

            @Override
            public String asString() {
                return this;
            }
        }

        public static class DateTime extends Value<LocalDateTime> {

            public DateTime(TypeQLToken.Predicate.Equality predicate, LocalDateTime value) {
                super(predicate, value);
                // validate precision of fractional seconds, which are stored as nanos in LocalDateTime
                int nanos = value.toLocalTime().getNano();
                final long nanosPerMilli = 1000000L;
                long remainder = nanos % nanosPerMilli;
                if (remainder != 0) {
                    throw TypeQLException.of(INVALID_CONSTRAINT_DATETIME_PRECISION.message(value));
                }
            }

            @Override
            public boolean isDateTime() {
                return true;
            }

            @Override
            public DateTime asDateTime() {
                return this;
            }
        }

        public static class Variable extends Value<ThingVariable<?>> {

            public Variable(TypeQLToken.Predicate.Equality predicate, UnboundVariable variable) {
                super(predicate, variable.toThing());
            }

            @Override
            public Set<BoundVariable> variables() {
                return set(value());
            }

            @Override
            public boolean isVariable() {
                return true;
            }

            @Override
            public Variable asVariable() {
                return this;
            }
        }
    }
}
