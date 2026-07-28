<?php

// Completeness fixture: one construct per grammar-legal variant of each
// field the php.{tags,calls,imports,complexity,types}.scm queries
// constrain, cross-referenced against arborium-php's node-types.json
// (v2.17.0). Every construct here is *expected to be captured*; see
// query_fixtures.rs's `php_*_completeness_*` tests for the matrix. A
// dedicated NEGATIVE section at the end holds near-miss constructs that
// must NOT match.

// --- imports.scm --------------------------------------------------------

use App\Models\User;                    // qualified_name import.path, no alias
use App\Models\Order as OrderModel;     // qualified_name import.path + alias
use Exception;                          // bare name import.path, no alias
use Throwable as T;                     // bare name import.path + alias
use App\Traits\{Loggable, Cacheable as Cache}; // grouped: two more namespace_use_clause entries
use function App\Helpers\formatDate;    // use function ...; (qualified_name, already covered)
use const App\Constants\MAX_SIZE;       // use const ...; (qualified_name, already covered)

require_once __DIR__ . '/bootstrap.php'; // require_once: string import.path
require 'config.php';                    // require: string import.path
include 'legacy.php';                    // include: string import.path
include_once 'once.php';                 // include_once: string import.path

trait GreetingTrait {
    public function greet(): string {
        return "hi";
    }
}

trait FarewellTrait {
    public function farewell(): string {
        return "bye";
    }
}

class Greeter {
    use GreetingTrait;             // use_declaration: bare name (trait composition, not a namespace import)
    use FarewellTrait, GreetingTrait; // multiple trait names as sibling children (no use_list wrapping)
}

// --- tags.scm / calls.scm: function_call_expression.function variants ---

function helperFn(): void {}

function callVariants(): void {
    helperFn();                          // function: name
    $fn = 'helperFn';
    $fn();                               // function: variable_name(name)
    \App\Collections\classify(1);        // function: qualified_name (defined in sample.php's namespace; parse-only)
    namespace\helperFn();                // function: relative_name
}

// --- tags.scm / calls.scm: scoped_call_expression.name variants ---------

class Toggle {
    public static function on(): void {}

    public function callViaSelf(): void {
        $method = 'on';
        self::on();                     // scoped_call name: name
        self::$method();                // scoped_call name: variable_name
        self::{$method}();              // scoped_call name: variable_name (braced form, same node kind)
    }
}

// --- tags.scm / calls.scm: member_call_expression.name variants ---------

class Dispatcher {
    public function on(): void {}

    public function dispatch(): void {
        $method = 'on';
        $this->on();                    // member_call name: name
        $this->$method();               // member_call name: variable_name
        $this->{$method}();             // member_call name: variable_name (braced form)
    }
}

// --- tags.scm / calls.scm: nullsafe_member_call_expression -------------

class Chain {
    public function next(): ?Chain {
        return null;
    }
}

function nullsafeCall(?Chain $c): void {
    $c?->next();                        // nullsafe_member_call name: name
}

// --- tags.scm: object_creation_expression (@reference.class) variants ---

class Widget {}

function constructorVariants(): void {
    new Widget();                       // object_creation: name
    new \App\Models\User();             // object_creation: qualified_name
    new namespace\Widget();             // object_creation: relative_name
    $cls = 'Widget';
    new $cls();                         // object_creation: variable_name
}

// --- tags.scm: base_clause / class_interface_clause variants ------------

interface Shape {}
interface Colored {}

class Circle extends Widget implements Shape, Colored {
    // base_clause: name (Widget); class_interface_clause: name, name (Shape, Colored)
}

// --- types.scm ------------------------------------------------------------

function typeVariants(
    int $a,                    // primitive_type
    Widget $b,                 // named_type -> name
    \App\Models\User $c,       // named_type -> qualified_name
    ?Widget $d,                // optional_type -> named_type (reached via unanchored named_type rule)
    int|string $e,             // union_type members (each reached via unanchored named_type/primitive_type rule)
    Shape&Colored $f,          // intersection_type members (same)
): void {}

function relativeTypeVariant(namespace\Widget $x): void {} // named_type -> relative_name

// --- complexity.scm --------------------------------------------------------

function complexityVariants(int $n): string {
    return match (true) {
        $n < 0 => "negative",   // match_conditional_expression (arm 1)
        $n === 0 => "zero",     // match_conditional_expression (arm 2)
        default => "positive",  // match_default_expression: NOT counted (see NEGATIVE section)
    };
}

function booleanOperatorVariants(bool $a, bool $b): bool {
    $r1 = $a && $b;  // binary_expression operator: "&&"
    $r2 = $a || $b;  // binary_expression operator: "||"
    $r3 = $a and $b; // binary_expression operator: "and"
    $r4 = $a or $b;  // binary_expression operator: "or"
    $r5 = $a xor $b; // binary_expression operator: "xor"
    return $r1 && $r2 && $r3 && $r4 && $r5;
}

// --- NEGATIVE cases: must not be captured -----------------------------------

class NegativeHolder {
    private int $field = 1;

    public function getField(): int {
        // A property read (`$this->field`) must never be captured as a
        // @call/@reference.call — it has no `arguments`/call shape at all,
        // it's a member_access_expression, a structurally different node.
        return $this->field;
    }
}

class AnonymousHolder {
    public function build(): object {
        // Anonymous class instantiation: object_creation_expression's first
        // child is `anonymous_class`, deliberately excluded from
        // php.tags.scm's @reference.class pattern (no name field exists;
        // capturing the whole class body as "name" text would be
        // fabricated, not just incomplete).
        return new class implements Shape {
            public function draw(): void {}
        };
    }
}

function negativeCases(): void {
    // A hash-like array literal is not a function call: must never be
    // captured as @call despite containing parenthesized-looking syntax.
    $arr = ['a' => 1, 'b' => 2];

    // A plain arithmetic binary expression must not be captured as
    // @complexity — only the specific boolean operators (&&, ||, and, or,
    // xor) are matched via the `operator` field, not the generic
    // binary_expression node kind.
    $sum = 1 + 2;

    // An arrow function and a closure are values (arrow_function /
    // anonymous_function), never `function_definition`/`method_declaration`
    // — must not appear as @definition.function/@definition.method.
    $fn = fn($x) => $x + 1;
    $closure = function ($x) {
        return $x;
    };
}
