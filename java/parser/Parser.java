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

package com.vaticle.typeql.lang.parser;

import com.vaticle.typedb.common.collection.Either;
import com.vaticle.typedb.common.collection.Pair;
import com.vaticle.typeql.grammar.TypeQLBaseVisitor;
import com.vaticle.typeql.grammar.TypeQLLexer;
import com.vaticle.typeql.grammar.TypeQLParser;
import com.vaticle.typeql.lang.common.TypeQLArg;
import com.vaticle.typeql.lang.common.TypeQLToken;
import com.vaticle.typeql.lang.common.exception.TypeQLException;
import com.vaticle.typeql.lang.pattern.Conjunction;
import com.vaticle.typeql.lang.pattern.Definable;
import com.vaticle.typeql.lang.pattern.Disjunction;
import com.vaticle.typeql.lang.pattern.Negation;
import com.vaticle.typeql.lang.pattern.Pattern;
import com.vaticle.typeql.lang.pattern.constraint.ThingConstraint;
import com.vaticle.typeql.lang.pattern.constraint.TypeConstraint;
import com.vaticle.typeql.lang.pattern.schema.Rule;
import com.vaticle.typeql.lang.pattern.variable.BoundVariable;
import com.vaticle.typeql.lang.pattern.variable.ConceptVariable;
import com.vaticle.typeql.lang.pattern.variable.ThingVariable;
import com.vaticle.typeql.lang.pattern.variable.TypeVariable;
import com.vaticle.typeql.lang.pattern.variable.UnboundVariable;
import com.vaticle.typeql.lang.query.TypeQLDefine;
import com.vaticle.typeql.lang.query.TypeQLDelete;
import com.vaticle.typeql.lang.query.TypeQLInsert;
import com.vaticle.typeql.lang.query.TypeQLMatch;
import com.vaticle.typeql.lang.query.TypeQLQuery;
import com.vaticle.typeql.lang.query.TypeQLUndefine;
import com.vaticle.typeql.lang.query.TypeQLUpdate;
import com.vaticle.typeql.lang.query.builder.Sortable;
import org.antlr.v4.runtime.ANTLRErrorStrategy;
import org.antlr.v4.runtime.BailErrorStrategy;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.antlr.v4.runtime.DefaultErrorStrategy;
import org.antlr.v4.runtime.ParserRuleContext;
import org.antlr.v4.runtime.atn.PredictionMode;
import org.antlr.v4.runtime.misc.ParseCancellationException;
import org.antlr.v4.runtime.tree.TerminalNode;

import javax.annotation.Nullable;
import java.time.LocalDate;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.time.format.DateTimeParseException;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.function.Function;
import java.util.stream.Stream;

import static com.vaticle.typedb.common.collection.Collections.pair;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.ILLEGAL_GRAMMAR;
import static com.vaticle.typeql.lang.common.exception.ErrorMessage.ILLEGAL_STATE;
import static com.vaticle.typeql.lang.common.util.Strings.unescapeRegex;
import static com.vaticle.typeql.lang.common.util.Strings.unescapeString;
import static com.vaticle.typeql.lang.common.util.Strings.unquoteString;
import static com.vaticle.typeql.lang.pattern.variable.UnboundVariable.hidden;
import static java.util.stream.Collectors.toList;
import static org.antlr.v4.runtime.atn.PredictionMode.LL_EXACT_AMBIG_DETECTION;
import static org.antlr.v4.runtime.atn.PredictionMode.SLL;

/**
 * TypeQL query string parser to produce TypeQL Java objects
 */
public class Parser extends TypeQLBaseVisitor {

    private static final Set<String> TYPEQL_KEYWORDS = getKeywords();

    private static Set<String> getKeywords() {
        HashSet<String> keywords = new HashSet<>();

        for (int i = 1; i <= TypeQLLexer.VOCABULARY.getMaxTokenType(); i++) {
            if (TypeQLLexer.VOCABULARY.getLiteralName(i) != null) {
                String name = TypeQLLexer.VOCABULARY.getLiteralName(i);
                keywords.add(name.replaceAll("'", ""));
            }
        }

        return Collections.unmodifiableSet(keywords);
    }

    public TypeQLLexer lexer(String string) {
        return new TypeQLLexer(CharStreams.fromString(string));
    }

    private <CONTEXT extends ParserRuleContext, RETURN> RETURN parse(
            String rawTypeQLString, Function<TypeQLParser, CONTEXT> rule, Function<CONTEXT, RETURN> visitor
    ) {
        if (rawTypeQLString == null) throw TypeQLException.of("Query String is NULL");
        String typeQLString = rawTypeQLString.stripTrailing();
        if (typeQLString.isEmpty()) throw TypeQLException.of("Query String is empty or blank");

        try {
            // BailErrorStrategy + SLL is a very fast parsing strategy for queries
            // that are expected to be correct. However, it may not be able to
            // provide detailed/useful error message, if at all.
            return visitor.apply(parseContext(rule, typeQLString, new BailErrorStrategy(), SLL, null));
        } catch (ParseCancellationException e) {
            // We parse the query one more time, with "strict strategy" :
            // DefaultErrorStrategy + LL_EXACT_AMBIG_DETECTION
            // This was not set to default parsing strategy, but it is useful
            // to produce detailed/useful error message
            ErrorListener errorListener = ErrorListener.of(typeQLString);
            parseContext(rule, typeQLString, new DefaultErrorStrategy(), LL_EXACT_AMBIG_DETECTION, errorListener);
            throw TypeQLException.of(errorListener.toString());
        }
    }

    private <CONTEXT extends ParserRuleContext> CONTEXT parseContext(
            Function<TypeQLParser, CONTEXT> rule,
            String typeQLString, ANTLRErrorStrategy errorHandlingStrategy, PredictionMode prediction,
            @Nullable ErrorListener errorListener
    ) {
        TypeQLLexer lexer = lexer(typeQLString);
        lexer.removeErrorListeners();
        if (errorListener != null) lexer.addErrorListener(errorListener);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        TypeQLParser parser = new TypeQLParser(tokens);
        parser.removeErrorListeners();
        if (errorListener != null) parser.addErrorListener(errorListener);
        parser.setErrorHandler(errorHandlingStrategy);
        parser.getInterpreter().setPredictionMode(prediction);
        return rule.apply(parser);
    }

    @SuppressWarnings("unchecked")
    public <T extends TypeQLQuery> T parseQueryEOF(String queryString) {
        return (T) parse(queryString, TypeQLParser::eof_query, this::visitEof_query);
    }

    @SuppressWarnings("unchecked")
    public <T extends TypeQLQuery> Stream<T> parseQueriesEOF(String queryString) {
        return (Stream<T>) parse(queryString, TypeQLParser::eof_queries, this::visitEof_queries);
    }

    public Pattern parsePatternEOF(String patternString) {
        return parse(patternString, TypeQLParser::eof_pattern, this::visitEof_pattern);
    }

    public List<? extends Pattern> parsePatternsEOF(String patternsString) {
        return parse(patternsString, TypeQLParser::eof_patterns, this::visitEof_patterns);
    }

    public List<Definable> parseDefinablesEOF(String definablesString) {
        return parse(definablesString, TypeQLParser::eof_definables, this::visitEof_definables);
    }

    public BoundVariable parseVariableEOF(String variableString) {
        return parse(variableString, TypeQLParser::eof_variable, this::visitEof_variable);
    }

    public String parseLabelEOF(String labelString) {
        return parse(labelString, TypeQLParser::eof_label, this::visitEof_label);
    }

    public Definable parseSchemaRuleEOF(String ruleString) {
        return parse(ruleString, TypeQLParser::eof_schema_rule, this::visitEof_schema_rule);
    }

    // GLOBAL HELPER METHODS ===================================================

    private UnboundVariable getVar(TerminalNode variable) {
        // Remove '$' prefix
        String name = variable.getSymbol().getText().substring(1);

        if (name.equals(TypeQLToken.Char.UNDERSCORE.toString())) {
            return UnboundVariable.anonymous();
        } else {
            return UnboundVariable.named(name);
        }
    }

    // PARSER VISITORS =========================================================

    @Override
    public TypeQLQuery visitEof_query(TypeQLParser.Eof_queryContext ctx) {
        return visitQuery(ctx.query());
    }

    @Override
    public Stream<? extends TypeQLQuery> visitEof_queries(TypeQLParser.Eof_queriesContext ctx) {
        return ctx.query().stream().map(this::visitQuery);
    }

    @Override
    public Pattern visitEof_pattern(TypeQLParser.Eof_patternContext ctx) {
        return visitPattern(ctx.pattern());
    }

    @Override
    public List<? extends Pattern> visitEof_patterns(TypeQLParser.Eof_patternsContext ctx) {
        return visitPatterns(ctx.patterns());
    }

    @Override
    public List<Definable> visitEof_definables(TypeQLParser.Eof_definablesContext ctx) {
        return ctx.definables().definable().stream().map(this::visitDefinable).collect(toList());
    }

    @Override
    public BoundVariable visitEof_variable(TypeQLParser.Eof_variableContext ctx) {
        return visitPattern_variable(ctx.pattern_variable());
    }

    @Override
    public String visitEof_label(TypeQLParser.Eof_labelContext ctx) {
        return ctx.label().getText();
    }

    @Override
    public Rule visitEof_schema_rule(TypeQLParser.Eof_schema_ruleContext ctx) {
        return visitSchema_rule(ctx.schema_rule());
    }

    // TYPEQL QUERIES ===========================================================

    @Override
    public TypeQLQuery visitQuery(TypeQLParser.QueryContext ctx) {
        if (ctx.query_define() != null) {
            return visitQuery_define(ctx.query_define());

        } else if (ctx.query_undefine() != null) {
            return visitQuery_undefine(ctx.query_undefine());

        } else if (ctx.query_insert() != null) {
            return visitQuery_insert(ctx.query_insert());

        } else if (ctx.query_delete() != null) {
            return visitQuery_delete(ctx.query_delete());
        } else if (ctx.query_update() != null) {
            return visitQuery_update(ctx.query_update());
        } else if (ctx.query_match() != null) {
            return visitQuery_match(ctx.query_match());

        } else if (ctx.query_match_aggregate() != null) {
            return visitQuery_match_aggregate(ctx.query_match_aggregate());

        } else if (ctx.query_match_group() != null) {
            return visitQuery_match_group(ctx.query_match_group());

        } else if (ctx.query_match_group_agg() != null) {
            return visitQuery_match_group_agg(ctx.query_match_group_agg());

        } else {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(ctx.getText()));
        }
    }

    @Override
    public TypeQLDefine visitQuery_define(TypeQLParser.Query_defineContext ctx) {
        List<Definable> definables = visitDefinables(ctx.definables());
        return new TypeQLDefine(definables);
    }

    @Override
    public TypeQLUndefine visitQuery_undefine(TypeQLParser.Query_undefineContext ctx) {
        List<Definable> definables = visitDefinables(ctx.definables());
        return new TypeQLUndefine(definables);
    }

    @Override
    public Rule visitSchema_rule(TypeQLParser.Schema_ruleContext ctx) {
        String label = ctx.label().getText();
        if (ctx.patterns() != null && ctx.variable_thing_any() != null) {
            List<? extends Pattern> when = visitPatterns(ctx.patterns());
            ThingVariable<?> then = visitVariable_thing_any(ctx.variable_thing_any());
            return new Rule(label, new Conjunction<>(when), then);
        } else {
            return new Rule(label);
        }
    }

    @Override
    public TypeQLInsert visitQuery_insert(TypeQLParser.Query_insertContext ctx) {
        if (ctx.patterns() != null) {
            return new TypeQLMatch.Unfiltered(visitPatterns(ctx.patterns()))
                    .insert(visitVariable_things(ctx.variable_things()));
        } else {
            return new TypeQLInsert(visitVariable_things(ctx.variable_things()));
        }
    }

    @Override
    public TypeQLDelete visitQuery_delete(TypeQLParser.Query_deleteContext ctx) {
        return new TypeQLMatch.Unfiltered(visitPatterns(ctx.patterns()))
                .delete(visitVariable_things(ctx.variable_things()));
    }

    @Override
    public TypeQLUpdate visitQuery_update(TypeQLParser.Query_updateContext ctx) {
        return visitQuery_delete(ctx.query_delete())
                .insert(visitVariable_things(ctx.variable_things()));
    }

    @Override
    public TypeQLMatch visitQuery_match(TypeQLParser.Query_matchContext ctx) {
        TypeQLMatch match = new TypeQLMatch.Unfiltered(visitPatterns(ctx.patterns()));

        if (ctx.modifiers() != null) {
            List<UnboundVariable> variables = new ArrayList<>();
            Sortable.Sorting sorting = null;
            Long offset = null, limit = null;

            if (ctx.modifiers().filter() != null) variables = this.visitFilter(ctx.modifiers().filter());
            if (ctx.modifiers().sort() != null) sorting = this.visitSort(ctx.modifiers().sort());
            if (ctx.modifiers().offset() != null) offset = getLong(ctx.modifiers().offset().LONG_());
            if (ctx.modifiers().limit() != null) limit = getLong(ctx.modifiers().limit().LONG_());
            match = new TypeQLMatch(match.conjunction(), variables, sorting, offset, limit);
        }

        return match;
    }

    /**
     * Visits the aggregate query node in the parsed syntax tree and builds the
     * appropriate aggregate query object
     *
     * @param ctx reference to the parsed aggregate query string
     * @return An AggregateQuery object
     */
    @Override
    public TypeQLMatch.Aggregate visitQuery_match_aggregate(TypeQLParser.Query_match_aggregateContext ctx) {
        TypeQLParser.Match_aggregateContext function = ctx.match_aggregate();

        return visitQuery_match(ctx.query_match()).aggregate(
                TypeQLToken.Aggregate.Method.of(function.aggregate_method().getText()),
                function.VAR_() != null ? getVar(function.VAR_()) : null
        );
    }

    @Override
    public TypeQLMatch.Group visitQuery_match_group(TypeQLParser.Query_match_groupContext ctx) {
        UnboundVariable var = getVar(ctx.match_group().VAR_());
        return visitQuery_match(ctx.query_match()).group(var);
    }

    @Override
    public TypeQLMatch.Group.Aggregate visitQuery_match_group_agg(TypeQLParser.Query_match_group_aggContext ctx) {
        UnboundVariable var = getVar(ctx.match_group().VAR_());
        TypeQLParser.Match_aggregateContext function = ctx.match_aggregate();

        return visitQuery_match(ctx.query_match()).group(var).aggregate(
                TypeQLToken.Aggregate.Method.of(function.aggregate_method().getText()),
                function.VAR_() != null ? getVar(function.VAR_()) : null
        );
    }

    // GET QUERY MODIFIERS ==========================================

    @Override
    public List<UnboundVariable> visitFilter(TypeQLParser.FilterContext ctx) {
        return ctx.VAR_().stream().map(this::getVar).collect(toList());
    }

    @Override
    public Sortable.Sorting visitSort(TypeQLParser.SortContext ctx) {
        List<Pair<UnboundVariable, TypeQLArg.Order>> sorting = ctx.var_order().stream().map(this::visitVar_order).collect(toList());
        return Sortable.Sorting.create(sorting);
    }

    @Override
    public Pair<UnboundVariable, TypeQLArg.Order> visitVar_order(TypeQLParser.Var_orderContext ctx) {
        return new Pair<>(getVar(ctx.VAR_()), ctx.ORDER_() == null ? null : TypeQLArg.Order.of(ctx.ORDER_().getText()));
    }


    // QUERY PATTERNS ==========================================================

    @Override
    public List<Pattern> visitPatterns(TypeQLParser.PatternsContext ctx) {
        return ctx.pattern().stream().map(this::visitPattern).collect(toList());
    }

    @Override
    public Pattern visitPattern(TypeQLParser.PatternContext ctx) {
        if (ctx.pattern_variable() != null) {
            return visitPattern_variable(ctx.pattern_variable());
        } else if (ctx.pattern_disjunction() != null) {
            return visitPattern_disjunction(ctx.pattern_disjunction());
        } else if (ctx.pattern_conjunction() != null) {
            return visitPattern_conjunction(ctx.pattern_conjunction());
        } else if (ctx.pattern_negation() != null) {
            return visitPattern_negation(ctx.pattern_negation());
        } else {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(ctx.getText()));
        }
    }

    @Override
    public Disjunction<? extends Pattern> visitPattern_disjunction(TypeQLParser.Pattern_disjunctionContext ctx) {
        List<Pattern> patterns = ctx.patterns().stream().map(patternsContext -> {
            List<Pattern> nested = visitPatterns(patternsContext);
            if (nested.size() > 1) return new Conjunction<>(nested);
            else return nested.get(0);
        }).collect(toList());

        assert patterns.size() > 1;

        return new Disjunction<>(patterns);
    }

    @Override
    public Conjunction<? extends Pattern> visitPattern_conjunction(TypeQLParser.Pattern_conjunctionContext ctx) {
        return new Conjunction<>(visitPatterns(ctx.patterns()));
    }

    @Override
    public Negation<? extends Pattern> visitPattern_negation(TypeQLParser.Pattern_negationContext ctx) {
        List<Pattern> patterns = visitPatterns(ctx.patterns());
        if (patterns.size() == 1) return new Negation<>(patterns.get(0));
        else return new Negation<>(new Conjunction<>(patterns));
    }

    // QUERY DEFINABLES ========================================================

    @Override
    public Definable visitDefinable(TypeQLParser.DefinableContext ctx) {
        if (ctx.variable_type() != null) {
            return visitVariable_type(ctx.variable_type());
        } else {
            return visitSchema_rule(ctx.schema_rule());
        }
    }

    @Override
    public List<Definable> visitDefinables(TypeQLParser.DefinablesContext ctx) {
        return ctx.definable().stream().map(this::visitDefinable).collect(toList());
    }


    // VARIABLE PATTERNS =======================================================

    @Override
    public BoundVariable visitPattern_variable(TypeQLParser.Pattern_variableContext ctx) {
        if (ctx.variable_thing_any() != null) {
            return this.visitVariable_thing_any(ctx.variable_thing_any());
        } else if (ctx.variable_type() != null) {
            return visitVariable_type(ctx.variable_type());
        } else if (ctx.variable_concept() != null) {
            return visitVariable_concept(ctx.variable_concept());
        } else {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(ctx.getText()));
        }
    }

    // CONCEPT VARIABLES =======================================================

    @Override
    public ConceptVariable visitVariable_concept(TypeQLParser.Variable_conceptContext ctx) {
        return getVar(ctx.VAR_(0)).is(getVar(ctx.VAR_(1)));
    }

    // TYPE VARIABLES ==========================================================

    @Override
    public TypeVariable visitVariable_type(TypeQLParser.Variable_typeContext ctx) {
        TypeVariable type = visitType_any(ctx.type_any()).apply(
                scopedLabel -> hidden().constrain(new TypeConstraint.Label(scopedLabel.first(), scopedLabel.second())),
                UnboundVariable::toType
        );

        for (TypeQLParser.Type_constraintContext constraint : ctx.type_constraint()) {
            if (constraint.ABSTRACT() != null) {
                type = type.isAbstract();
            } else if (constraint.SUB_() != null) {
                TypeQLToken.Constraint sub = TypeQLToken.Constraint.of(constraint.SUB_().getText());
                type = type.constrain(new TypeConstraint.Sub(visitType_any(constraint.type_any()), sub == TypeQLToken.Constraint.SUBX));
            } else if (constraint.OWNS() != null) {
                Either<String, UnboundVariable> overridden = constraint.AS() == null ? null : visitType(constraint.type(1));
                type = type.constrain(new TypeConstraint.Owns(visitType(constraint.type(0)), overridden, visitAnnotations_owns(constraint.annotations_owns())));
            } else if (constraint.PLAYS() != null) {
                Either<String, UnboundVariable> overridden = constraint.AS() == null ? null : visitType(constraint.type(0));
                type = type.constrain(new TypeConstraint.Plays(visitType_scoped(constraint.type_scoped()), overridden));
            } else if (constraint.RELATES() != null) {
                Either<String, UnboundVariable> overridden = constraint.AS() == null ? null : visitType(constraint.type(1));
                type = type.constrain(new TypeConstraint.Relates(visitType(constraint.type(0)), overridden));
            } else if (constraint.VALUE() != null) {
                type = type.value(TypeQLArg.ValueType.of(constraint.value_type().getText()));
            } else if (constraint.REGEX() != null) {
                type = type.regex(getRegex(constraint.STRING_()));
            } else if (constraint.TYPE() != null) {
                Pair<String, String> scopedLabel = visitLabel_any(constraint.label_any());
                type = type.constrain(new TypeConstraint.Label(scopedLabel.first(), scopedLabel.second()));
            } else {
                throw TypeQLException.of(ILLEGAL_GRAMMAR.message(constraint.getText()));
            }
        }

        return type;
    }

    @Override
    public TypeQLToken.Annotation[] visitAnnotations_owns(TypeQLParser.Annotations_ownsContext ctx) {
        // precompute array length to avoid double allocation of arrays
        int count = 0;
        if (ctx.ANNOTATION_KEY() != null) count++;
        if (ctx.ANNOTATION_UNIQUE() != null) count++;
        TypeQLToken.Annotation[] annotations = new TypeQLToken.Annotation[count];
        int index = 0;
        if (ctx.ANNOTATION_KEY() != null) {
            annotations[index] = parseAnnotation(ctx.ANNOTATION_KEY());
            index++;
        }
        if (ctx.ANNOTATION_UNIQUE() != null) {
            annotations[index] = parseAnnotation(ctx.ANNOTATION_UNIQUE());
            index++;
        }
        assert index == count;
        return annotations;
    }

    private TypeQLToken.Annotation parseAnnotation(TerminalNode terminalNode) {
        assert !terminalNode.getText().isEmpty() && terminalNode.getText().startsWith(TypeQLToken.Char.AT.toString());
        return TypeQLToken.Annotation.of(terminalNode.getText().substring(1));
    }

    // THING VARIABLES =========================================================

    @Override
    public List<ThingVariable<?>> visitVariable_things(TypeQLParser.Variable_thingsContext ctx) {
        return ctx.variable_thing_any().stream().map(this::visitVariable_thing_any).collect(toList());
    }

    @Override
    public ThingVariable<?> visitVariable_thing_any(TypeQLParser.Variable_thing_anyContext ctx) {
        if (ctx.variable_thing() != null) {
            return this.visitVariable_thing(ctx.variable_thing());
        } else if (ctx.variable_relation() != null) {
            return this.visitVariable_relation(ctx.variable_relation());
        } else if (ctx.variable_attribute() != null) {
            return this.visitVariable_attribute(ctx.variable_attribute());
        } else {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(ctx.getText()));
        }
    }

    @Override
    public ThingVariable.Thing visitVariable_thing(TypeQLParser.Variable_thingContext ctx) {
        UnboundVariable unbound = getVar(ctx.VAR_());
        ThingVariable.Thing thing = null;

        if (ctx.ISA_() != null) {
            thing = unbound.constrain(getIsaConstraint(ctx.ISA_(), ctx.type()));
        } else if (ctx.IID() != null) {
            thing = unbound.iid(ctx.IID_().getText());
        }

        if (ctx.attributes() != null) {
            for (ThingConstraint.Has hasAttribute : visitAttributes(ctx.attributes())) {
                if (thing == null) thing = unbound.constrain(hasAttribute);
                else thing = thing.constrain(hasAttribute);
            }
        }
        return thing;
    }

    @Override
    public ThingVariable.Relation visitVariable_relation(TypeQLParser.Variable_relationContext ctx) {
        UnboundVariable unbound;
        if (ctx.VAR_() != null) unbound = getVar(ctx.VAR_());
        else unbound = hidden();

        ThingVariable.Relation relation = unbound.constrain(visitRelation(ctx.relation()));
        if (ctx.ISA_() != null) relation = relation.constrain(getIsaConstraint(ctx.ISA_(), ctx.type()));

        if (ctx.attributes() != null) {
            for (ThingConstraint.Has hasAttribute : visitAttributes(ctx.attributes())) {
                relation = relation.constrain(hasAttribute);
            }
        }
        return relation;
    }

    @Override
    public ThingVariable.Attribute visitVariable_attribute(TypeQLParser.Variable_attributeContext ctx) {
        UnboundVariable unbound;
        if (ctx.VAR_() != null) unbound = getVar(ctx.VAR_());
        else unbound = hidden();

        ThingVariable.Attribute attribute = unbound.constrain(visitPredicate(ctx.predicate()));
        if (ctx.ISA_() != null) attribute = attribute.constrain(getIsaConstraint(ctx.ISA_(), ctx.type()));

        if (ctx.attributes() != null) {
            for (ThingConstraint.Has hasAttribute : visitAttributes(ctx.attributes())) {
                attribute = attribute.constrain(hasAttribute);
            }
        }
        return attribute;
    }

    private ThingConstraint.Isa getIsaConstraint(TerminalNode isaToken, TypeQLParser.TypeContext ctx) {
        TypeQLToken.Constraint isa = TypeQLToken.Constraint.of(isaToken.getText());

        if (isa != null && isa.equals(TypeQLToken.Constraint.ISA)) {
            return new ThingConstraint.Isa(visitType(ctx), false);
        } else if (isa != null && isa.equals(TypeQLToken.Constraint.ISAX)) {
            return new ThingConstraint.Isa(visitType(ctx), true);
        } else {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(ctx.getText()));
        }
    }

    // ATTRIBUTE STATEMENT CONSTRUCT ===============================================

    @Override
    public List<ThingConstraint.Has> visitAttributes(TypeQLParser.AttributesContext ctx) {
        return ctx.attribute().stream().map(this::visitAttribute).collect(toList());
    }

    @Override
    public ThingConstraint.Has visitAttribute(TypeQLParser.AttributeContext ctx) {
        if (ctx.label() != null) {
            if (ctx.VAR_() != null) return new ThingConstraint.Has(ctx.label().getText(), getVar(ctx.VAR_()));
            if (ctx.predicate() != null)
                return new ThingConstraint.Has(ctx.label().getText(), visitPredicate(ctx.predicate()));
        } else if (ctx.VAR_() != null) return new ThingConstraint.Has(getVar(ctx.VAR_()));
        throw TypeQLException.of(ILLEGAL_GRAMMAR.message(ctx.getText()));
    }

    // RELATION STATEMENT CONSTRUCT ============================================

    @Override
    public ThingConstraint.Relation visitRelation(TypeQLParser.RelationContext ctx) {
        List<ThingConstraint.Relation.RolePlayer> rolePlayers = new ArrayList<>();

        for (TypeQLParser.Role_playerContext rolePlayerCtx : ctx.role_player()) {
            UnboundVariable player = getVar(rolePlayerCtx.player().VAR_());
            if (rolePlayerCtx.type() != null) {
                Either<String, UnboundVariable> roleType = visitType(rolePlayerCtx.type());
                rolePlayers.add(new ThingConstraint.Relation.RolePlayer(roleType, player));
            } else {
                rolePlayers.add(new ThingConstraint.Relation.RolePlayer(player));
            }
        }
        return new ThingConstraint.Relation(rolePlayers);
    }

    // TYPE, LABEL, AND IDENTIFIER CONSTRUCTS ==================================

    @Override
    public Either<Pair<String, String>, UnboundVariable> visitType_any(TypeQLParser.Type_anyContext ctx) {
        if (ctx.VAR_() != null) return Either.second(getVar(ctx.VAR_()));
        else if (ctx.type() != null)
            return visitType(ctx.type()).apply(s -> Either.first(pair(null, s)), Either::second);
        else if (ctx.type_scoped() != null) return visitType_scoped(ctx.type_scoped());
        else return null;
    }

    @Override
    public Either<Pair<String, String>, UnboundVariable> visitType_scoped(TypeQLParser.Type_scopedContext ctx) {
        if (ctx.label_scoped() != null) return Either.first(visitLabel_scoped(ctx.label_scoped()));
        else if (ctx.VAR_() != null) return Either.second(getVar(ctx.VAR_()));
        else return null;
    }

    @Override
    public Either<String, UnboundVariable> visitType(TypeQLParser.TypeContext ctx) {
        if (ctx.label() != null) return Either.first(ctx.label().getText());
        else if (ctx.VAR_() != null) return Either.second(getVar(ctx.VAR_()));
        else return null;
    }

    @Override
    public Pair<String, String> visitLabel_any(TypeQLParser.Label_anyContext ctx) {
        if (ctx.label() != null) return pair(null, ctx.label().getText());
        else if (ctx.label_scoped() != null) return visitLabel_scoped(ctx.label_scoped());
        else return null;
    }

    @Override
    public Pair<String, String> visitLabel_scoped(TypeQLParser.Label_scopedContext ctx) {
        String[] scopedLabel = ctx.getText().split(":");
        return pair(scopedLabel[0], scopedLabel[1]);
    }

    // ATTRIBUTE OPERATION CONSTRUCTS ==========================================

    @Override
    public ThingConstraint.Value<?> visitPredicate(TypeQLParser.PredicateContext ctx) {
        TypeQLToken.Predicate predicate;
        Object value;

        if (ctx.value() != null) {
            predicate = TypeQLToken.Predicate.Equality.EQ;
            value = visitValue(ctx.value());
        } else if (ctx.predicate_equality() != null) {
            predicate = TypeQLToken.Predicate.Equality.of(ctx.predicate_equality().getText());
            if (ctx.predicate_value().value() != null) value = visitValue(ctx.predicate_value().value());
            else if (ctx.predicate_value().VAR_() != null) value = getVar(ctx.predicate_value().VAR_());
            else throw TypeQLException.of(ILLEGAL_STATE);
        } else if (ctx.predicate_substring() != null) {
            predicate = TypeQLToken.Predicate.SubString.of(ctx.predicate_substring().getText());
            if (ctx.predicate_substring().LIKE() != null) value = getRegex(ctx.STRING_());
            else value = getString(ctx.STRING_());
        } else throw TypeQLException.of(ILLEGAL_STATE);

        assert predicate != null;

        if (value instanceof Long) {
            return new ThingConstraint.Value.Long(predicate.asEquality(), (Long) value);
        } else if (value instanceof Double) {
            return new ThingConstraint.Value.Double(predicate.asEquality(), (Double) value);
        } else if (value instanceof Boolean) {
            return new ThingConstraint.Value.Boolean(predicate.asEquality(), (Boolean) value);
        } else if (value instanceof String) {
            return new ThingConstraint.Value.String(predicate, (String) value);
        } else if (value instanceof LocalDateTime) {
            return new ThingConstraint.Value.DateTime(predicate.asEquality(), (LocalDateTime) value);
        } else if (value instanceof UnboundVariable) {
            return new ThingConstraint.Value.Variable(predicate.asEquality(), (UnboundVariable) value);
        } else {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(ctx.getText()));
        }
    }

    // LITERAL INPUT VALUES ====================================================

    public String getRegex(TerminalNode string) {
        return unescapeRegex(unescapeString(unquoteString(string.getText())));
    }

    @Override
    public TypeQLArg.ValueType visitValue_type(TypeQLParser.Value_typeContext valueClass) {
        if (valueClass.BOOLEAN() != null) {
            return TypeQLArg.ValueType.BOOLEAN;
        } else if (valueClass.DATETIME() != null) {
            return TypeQLArg.ValueType.DATETIME;
        } else if (valueClass.DOUBLE() != null) {
            return TypeQLArg.ValueType.DOUBLE;
        } else if (valueClass.LONG() != null) {
            return TypeQLArg.ValueType.LONG;
        } else if (valueClass.STRING() != null) {
            return TypeQLArg.ValueType.STRING;
        } else {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(valueClass));
        }
    }

    @Override
    public Object visitValue(TypeQLParser.ValueContext ctx) {
        if (ctx.STRING_() != null) {
            return getString(ctx.STRING_());

        } else if (ctx.LONG_() != null) {
            return getLong(ctx.LONG_());

        } else if (ctx.DOUBLE_() != null) {
            return getDouble(ctx.DOUBLE_());

        } else if (ctx.BOOLEAN_() != null) {
            return getBoolean(ctx.BOOLEAN_());

        } else if (ctx.DATE_() != null) {
            return getDate(ctx.DATE_());

        } else if (ctx.DATETIME_() != null) {
            return getDateTime(ctx.DATETIME_());

        } else {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(ctx.getText()));
        }
    }

    private String getString(TerminalNode string) {
        String str = string.getText();
        assert str.length() >= 2;
        TypeQLToken.Char start = TypeQLToken.Char.of(str.substring(0, 1));
        TypeQLToken.Char end = TypeQLToken.Char.of(str.substring(str.length() - 1));
        assert start != null && end != null;
        assert start.equals(TypeQLToken.Char.QUOTE_DOUBLE) || start.equals(TypeQLToken.Char.QUOTE_SINGLE);
        assert end.equals(TypeQLToken.Char.QUOTE_DOUBLE) || end.equals(TypeQLToken.Char.QUOTE_SINGLE);
        // Remove surrounding quotes and backslash escapes
        return unescapeString(unquoteString(str));
    }

    private long getLong(TerminalNode number) {
        try {
            return Long.parseLong(number.getText());
        } catch (NumberFormatException e) {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(number.getText()));
        }
    }

    private double getDouble(TerminalNode real) {
        try {
            return Double.parseDouble(real.getText());
        } catch (NumberFormatException e) {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(real.getText()));
        }
    }

    private boolean getBoolean(TerminalNode bool) {
        TypeQLToken.Literal literal = TypeQLToken.Literal.of(bool.getText());

        if (literal != null && literal.equals(TypeQLToken.Literal.TRUE)) {
            return true;

        } else if (literal != null && literal.equals(TypeQLToken.Literal.FALSE)) {
            return false;

        } else {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(bool.getText()));
        }
    }

    private LocalDateTime getDate(TerminalNode date) {
        try {
            return LocalDate.parse(date.getText(), DateTimeFormatter.ISO_LOCAL_DATE).atStartOfDay();
        } catch (DateTimeParseException e) {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(date.getText()));
        }
    }

    private LocalDateTime getDateTime(TerminalNode dateTime) {
        try {
            return LocalDateTime.parse(dateTime.getText(), DateTimeFormatter.ISO_LOCAL_DATE_TIME);
        } catch (DateTimeParseException e) {
            throw TypeQLException.of(ILLEGAL_GRAMMAR.message(dateTime.getText()));
        }
    }
}
