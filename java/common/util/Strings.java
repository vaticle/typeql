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

package com.vaticle.typeql.lang.common.util;

import java.text.DecimalFormat;
import java.text.DecimalFormatSymbols;
import java.util.Arrays;
import java.util.Locale;

import static com.vaticle.typeql.lang.common.TypeQLToken.Char.INDENTATION;
import static com.vaticle.typeql.lang.common.TypeQLToken.Char.NEW_LINE;

public class Strings {

    public static String unescapeRegex(String regex) {
        return regex.replaceAll("\\\\/", "/");
    }

    public static String escapeRegex(String regex) {
        return regex.replaceAll("/", "\\\\/");
    }

    public static String unescapeString(String string) {
        return string.replaceAll("\\\\(.)", "$1");
    }

    public static String escapeString(String string) {
        return string.replaceAll("(\\\\|\")", "\\\\$1");
    }

    /**
     * @param string a string to quote and escape
     * @return a string, surrounded with double quotes and escaped
     */
    public static String quoteString(String string) {
        return "\"" + string + "\"";
    }

    public static String unquoteString(String string) {
        return string.substring(1, string.length() - 1);
    }

    public static String indent(Object object) {
        return indent(object.toString());
    }

    public static String indent(String string) {
        return Arrays.stream(string.split("\n")).map(s -> INDENTATION + s).collect(NEW_LINE.joiner());
    }

    /**
     * @param value a value in the graph
     * @return the string representation of the value (using quotes if it is already a string)
     */
    public static String valueToString(Object value) {
        if (value instanceof String) {
            return quoteString((String) value);
        } else if (value instanceof Double) {
            DecimalFormat df = new DecimalFormat("#", DecimalFormatSymbols.getInstance(Locale.ENGLISH));
            df.setMinimumFractionDigits(1);
            df.setMaximumFractionDigits(12);
            df.setMinimumIntegerDigits(1);
            return df.format(value);
        } else {
            return value.toString();
        }
    }
}
