// Completeness fixture: one construct per grammar-legal variant of each field
// the java.{tags,calls,imports,complexity,types}.scm queries constrain,
// cross-referenced against arborium-java 2.17.0's node-types.json. Every
// construct here is *expected to be captured*; see query_fixtures.rs
// `java_*_completeness_*` tests for the matrix.
//
// This file also carries a NEGATIVE section with deliberate near-miss
// constructs that must NOT be captured by the query under test, to guard
// against over-broad patterns.

import Bare; // import_declaration argument: bare `identifier` (no package) —
             // rare/non-idiomatic (default-package imports aren't legal Java)
             // but grammar-legal; see java.imports.scm.
import java.util.ArrayList; // import_declaration argument: scoped_identifier
import java.util.List; // plain multi-segment import
import java.util.*; // import_declaration: scoped_identifier + asterisk (wildcard)
import static java.lang.Math.PI; // import static pkg.Class.member;
import static java.lang.Math.*; // import static pkg.Class.*;

// --- object_creation_expression.type variants -------------------------------

class NewVariants {
    void plainNew() {
        new Object(); // type: type_identifier
    }

    void genericNew() {
        new ArrayList<String>(); // type: generic_type -> type_identifier
    }

    void genericDiamondNew() {
        new ArrayList<>(); // type: generic_type -> type_identifier (diamond)
    }

    void scopedNew() {
        new java.util.Date(); // type: scoped_type_identifier
    }

    void genericScopedNew() {
        new java.util.HashMap<String, Integer>(); // type: generic_type -> scoped_type_identifier
    }
}

// --- superclass variants ----------------------------------------------------

class PlainBase {}

class GenericBase<T> {}

class ExtendsPlain extends PlainBase {} // superclass: type_identifier

class ExtendsGeneric extends GenericBase<String> {} // superclass: generic_type -> type_identifier

class ExtendsScoped extends java.util.AbstractList<String> {
    // superclass: generic_type -> scoped_type_identifier
    @Override
    public String get(int index) {
        return null;
    }

    @Override
    public int size() {
        return 0;
    }
}

// --- type_list (implements) variants ----------------------------------------

interface PlainIface {}

interface GenericIface<T> {}

class ImplementsPlain implements PlainIface {} // type_list: type_identifier

class ImplementsGeneric implements Comparable<ImplementsGeneric> {
    // type_list: generic_type -> type_identifier
    @Override
    public int compareTo(ImplementsGeneric o) {
        return 0;
    }
}

class ImplementsScoped implements java.io.Serializable {} // type_list: scoped_type_identifier

class ImplementsGenericScoped implements java.util.Comparator<String> {
    // type_list: generic_type -> scoped_type_identifier
    @Override
    public int compare(String a, String b) {
        return 0;
    }
}

// --- Type-defining declaration variants (java.tags.scm / java.types.scm) ---

class PlainClass {} // definition.class

interface PlainInterface {} // definition.interface

enum PlainEnum {
    // definition.enum; enum constant with constructor argument
    ONE(1),
    TWO(2);

    final int value;

    PlainEnum(int value) {
        this.value = value;
    }
}

record PlainRecord(int a, int b) {} // definition.class (record_declaration)

@interface PlainAnnotation {
    // definition.interface (annotation_type_declaration)
    String value();
}

// --- method_invocation variants (java.calls.scm) ----------------------------

class CallVariants {
    void plainCall() {
        identity(1); // no object field
    }

    void qualifiedCall() {
        Math.abs(-1); // object: identifier (Class.staticMethod())
    }

    void chainedCall() {
        String s = "x";
        s.trim().toUpperCase().length(); // object: method_invocation (chained)
    }

    int identity(int x) {
        return x;
    }
}

// --- Generic method / bounded type parameter --------------------------------

class GenericMethodHolder {
    <T extends Comparable<T>> T max(T a, T b) {
        return a.compareTo(b) >= 0 ? a : b;
    }
}

// --- Varargs -----------------------------------------------------------------

class VarargsHolder {
    void sum(int... nums) {
        int total = 0;
        for (int n : nums) {
            total += n;
        }
    }
}

// --- try-with-resources / switch (arrow form) / complexity variants --------

class ComplexityVariants {
    void tryWithResources() {
        try (var r = new java.io.StringReader("x")) {
            r.read();
        } catch (Exception e) {
            e.printStackTrace();
        } finally {
            System.out.println("done");
        }
    }

    String switchArrow(int n) {
        return switch (n) {
            case 1 -> "one";
            case 2 -> "two";
            default -> "other";
        };
    }

    void loops(int n) {
        for (int i = 0; i < n; i++) {
        }
        int i = 0;
        while (i < n) {
            i++;
        }
        do {
            i--;
        } while (i > 0);
        for (int x : new int[] {1, 2, 3}) {
        }
    }
}

// --- NEGATIVE cases: must not be captured as calls/definitions --------------

class NegativeHolder {
    int field;

    void negativeCases() {
        // A lambda is not a method_declaration; must never appear as
        // @definition.method / @definition.function in tags.
        Runnable lambdaBinding = () -> System.out.println("run");

        // A method reference is not an invocation; must never appear in
        // java.calls.scm's @call captures (it's a functional value, not a call).
        java.util.function.Supplier<String> methodRef = NegativeHolder::staticMethod;

        // Bare field access with no call parens must never appear as a call.
        int read = this.field;

        // Field write is not a "call".
        this.field = 5;
    }

    static String staticMethod() {
        return "x";
    }
}
