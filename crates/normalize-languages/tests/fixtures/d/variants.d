// Completeness matrix for D's tags/calls/imports/types/complexity/cfg/refactor
// queries. One small, commented construct per node-type variant found via
// node-types.json (arborium-d 2.17.0), plus a NEGATIVE section of near-miss
// constructs that must NOT produce the capture they superficially resemble.
// See crates/normalize-languages/src/queries/d.*.scm for the query files
// this exercises.

module variants;

import io = std.stdio;              // import_declaration + module_alias_identifier (@import.alias)
import std.math : sqrt, pow;        // import_declaration + import_bindings (@import.path only)

// --- tags: @definition.class -----------------------------------------------

class PlainClass {                  // class_declaration
    int x;
}

class GenericClass(T) {             // class_template_declaration (distinct node type)
    T value;
}

struct PlainStruct {                // struct_declaration
    int x;
}

struct GenericStruct(T) {           // struct_template_declaration (distinct node type)
    T value;
}

union PlainUnion {                  // union_declaration
    int x;
    float y;
}

union GenericUnion(T) {             // union_template_declaration (distinct node type)
    T value;
}

// --- tags: @definition.interface -------------------------------------------

interface PlainInterface {          // interface_declaration
    void doIt();
}

interface GenericInterface(T) {     // interface_template_declaration (distinct node type)
    void doIt(T x);
}

// --- tags: @definition.type -------------------------------------------------

enum Color { Red, Green, Blue }     // enum_declaration

// --- tags: @definition.function ---------------------------------------------

int plainFunc(int x) {              // func_declaration > func_declarator > identifier
    return x;
}

auto autoFunc(int x) {              // auto_func_declaration (distinct node type, no func_declarator)
    return x;
}

// --- types: @type.reference (structural-position match, not "any qualified_identifier") ---

PlainClass globalVarType;           // var_declarations > qualified_identifier (direct child)
std.math.PI globalNestedType;       // var_declarations > qualified_identifier chain (outermost only)

void typeSites(PlainClass paramType) { // parameter > qualified_identifier (direct child)
    PlainClass localVarType;        // var_declarations (nested in declaration_statement)
    auto casted = cast(PlainClass) paramType; // cast_expression > type > qualified_identifier
    auto made = new PlainClass();   // new_expression > type > qualified_identifier
}

PlainClass returnTypeSite() {       // func_declaration > qualified_identifier (return type, direct child)
    return new PlainClass();
}

alias AliasedType = PlainClass;     // alias_declaration > alias_assignments > alias_assignment > type > qualified_identifier

// --- calls: @call -------------------------------------------------------------

void callSites() {
    plainFunc(1);                   // postfix_expression > qualified_identifier (bare call)
    io.writeln("x");                // postfix_expression > qualified_identifier (member-call chain)
    GenericClass!int();             // postfix_expression > qualified_identifier (template instance call)
}

// --- complexity / cfg / refactor: switch vs final switch ---------------------

int plainSwitch(int n) {
    switch (n) {                    // switch_statement
        case 0:
            return 0;
        default:
            return 1;
    }
}

int finalSwitch(Color c) {
    final switch (c) {              // final_switch_statement (distinct node type from switch_statement)
        case Color.Red: return 0;
        case Color.Green: return 1;
        case Color.Blue: return 2;
    }
}

// --- NEGATIVE: constructs that must NOT match ---------------------------------

void negatives() {
    // Bare qualified_identifier used as a call target must NOT match @type.reference
    // (this was the original bug: `(qualified_identifier) @type.reference` matched
    // every call and member access, not just type positions).
    plainFunc(1);
    io.writeln("not a type");

    // A default-value call expression inside a parameter list must not be
    // mistaken for the parameter's type.
    helper(1);
}

void helper(int x, int y = plainFunc(1)) {
}
