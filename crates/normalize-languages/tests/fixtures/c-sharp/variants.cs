// Completeness fixture: one construct per grammar-legal variant of each field
// the c-sharp.{tags,calls,imports,complexity,types}.scm queries constrain,
// cross-referenced against arborium-c-sharp 2.17.0's node-types.json. Every
// construct here is *expected to be captured*; see query_fixtures.rs
// `csharp_*_completeness_*` tests for the matrix.
//
// This file also carries a NEGATIVE section with deliberate near-miss
// constructs that must NOT be captured by the query under test, to guard
// against over-broad patterns.

using Bare; // using_directive path: bare identifier
using System.Collections.Generic; // using_directive path: qualified_name
using static System.Math; // using static <qualified_name>
using static Wrapper<int>; // using static <bare generic_name>
using Sys = System; // using_directive alias -> identifier
using SysColl = System.Collections.Generic; // using_directive alias -> qualified_name
using MyList = List<int>; // using_directive alias -> bare generic_name
using global::System; // using_directive path: bare alias_qualified_name

namespace Variants
{
    class Wrapper<T> { }

    // --- base_list variants (c-sharp.tags.scm / types.scm) ---------------------

    class PlainBase { }

    class GenericBase<T> { }

    interface IPlainIface { }

    interface IGenericIface<T> { }

    // base_list: identifier (plain base class)
    class ExtendsPlain : PlainBase { }

    // base_list: generic_name -> identifier (generic base class)
    class ExtendsGeneric : GenericBase<string> { }

    // base_list: qualified_name -> identifier (path-qualified base class)
    class ExtendsScoped : System.Exception { }

    // base_list: identifier interface (plain implements)
    class ImplementsPlain : IPlainIface { }

    // base_list: generic_name -> identifier interface (generic implements)
    class ImplementsGeneric : IGenericIface<string> { }

    // base_list: qualified_name -> identifier interface (path-qualified implements)
    class ImplementsScoped : System.IDisposable
    {
        public void Dispose() { }
    }

    // base_list: multiple entries (base class + 2 interfaces) in one clause
    class MultiBase : PlainBase, IPlainIface, IGenericIface<int> { }

    // primary_constructor_base_type: identifier (record primary-ctor base)
    record RecordBase(int A);
    record RecordDerivedPlain(int A, int B) : RecordBase(A);

    // primary_constructor_base_type: generic_name -> identifier
    record RecordBaseGeneric<T>(T A);
    record RecordDerivedGeneric(int A) : RecordBaseGeneric<int>(A);

    // --- Type-defining declaration variants (tags.scm / types.scm) -------------

    class PlainClass { } // definition.class / definition.type

    struct PlainStruct { } // definition.class (struct maps to class kind) / definition.type

    interface PlainInterface { } // definition.interface / definition.type

    enum PlainEnum { A, B } // definition.enum / definition.type

    record PlainRecord(int X); // definition.class (record_declaration) / definition.type

    namespace Nested // namespace_declaration: definition.module
    {
        class Inner { }
    }

    // --- object_creation_expression.type variants (tags.scm) -------------------

    class NewVariants
    {
        void PlainNew()
        {
            // type: identifier. NOTE: `new object()`/`new int()` would use
            // `predefined_type` (a distinct keyword-type node), not
            // `identifier` — a user-defined type name is required to
            // actually exercise the identifier variant.
            var p = new PlainClass();
        }

        void GenericNew()
        {
            var l = new List<int>(); // type: generic_name -> identifier
        }

        void ScopedNew()
        {
            var d = new System.Text.StringBuilder(); // type: qualified_name -> identifier
        }
    }

    // --- invocation_expression.function / member_access variants (calls.scm) --

    class CallVariants
    {
        void PlainCall()
        {
            Identity(1); // function: identifier (no qualifier)
        }

        void GenericCall()
        {
            GenericIdentity<int>(1); // function: generic_name -> identifier
        }

        void QualifiedCall()
        {
            System.Console.WriteLine("x"); // function: member_access_expression, name: identifier
        }

        void QualifiedGenericCall(List<object> xs)
        {
            xs.OfType<int>(); // function: member_access_expression, name: generic_name -> identifier
        }

        void ChainedCall(string s)
        {
            s.Trim().ToUpper(); // qualifier: method_invocation (chained)
        }

        void ConditionalCall(string? s)
        {
            s?.Trim(); // function: conditional_access_expression, member_binding_expression.name: identifier
        }

        void ConditionalGenericCall(List<object>? xs)
        {
            xs?.OfType<int>(); // conditional_access_expression, member_binding_expression.name: generic_name
        }

        int Identity(int x) => x;
        T GenericIdentity<T>(T x) => x;
    }

    // --- constructor_initializer variants (calls.scm / tags.scm) ---------------

    class CtorBase
    {
        public CtorBase() { }
        public CtorBase(int x) { }
    }

    class CtorDerived : CtorBase
    {
        public CtorDerived() : base(1) { } // constructor_initializer: "base"
        public CtorDerived(int x) : this() { } // constructor_initializer: "this"
    }

    // --- types.scm field-position variants --------------------------------------

    class TypePositions
    {
        // NOTE: `int`/`object`/`string`/`bool` are `predefined_type` (a
        // distinct keyword-type node), NOT `identifier` — every "identifier
        // variant" construct below deliberately uses a user-defined type
        // name (`PlainClass`) rather than a builtin, since a builtin would
        // silently (and correctly) fail to exercise the identifier pattern.

        List<int> field1; // variable_declaration.type: generic_name
        System.Text.StringBuilder field2; // variable_declaration.type: qualified_name
        PlainClass field3; // variable_declaration.type: identifier
        PlainClass? field4; // variable_declaration.type: nullable_type -> identifier
        List<int>? field5; // variable_declaration.type: nullable_type -> generic_name

        void Param(PlainClass a, List<int> b, System.Text.StringBuilder c, PlainClass? d) { } // parameter.type variants

        PlainClass ReturnPlain() => new PlainClass(); // method_declaration.returns: identifier
        List<int> ReturnGeneric() => new List<int>(); // method_declaration.returns: generic_name
        System.Text.StringBuilder ReturnScoped() => new(); // method_declaration.returns: qualified_name

        void LocalFn()
        {
            PlainClass LocalPlain() => new PlainClass(); // local_function_statement.type: identifier
            List<int> LocalGeneric() => new List<int>(); // local_function_statement.type: generic_name
        }

        PlainClass PropPlain { get; set; } // property_declaration.type: identifier
        List<int> PropGeneric { get; set; } // property_declaration.type: generic_name

        void Foreach(List<PlainClass> xs)
        {
            foreach (PlainClass x in xs) // foreach_statement.type: identifier
            {
            }
        }

        void TryCatch()
        {
            try { }
            catch (System.Exception ex) // catch_declaration.type: qualified_name
            {
            }
        }

        void CastAndPattern(object o)
        {
            var x = (PlainClass)o; // cast_expression.type: identifier
            var y = o is List<int> list; // is_expression.right: generic_name
            var z = o as PlainClass; // as_expression.right: identifier
        }
    }

    // --- switch_expression_arm variants (complexity.scm) -----------------------

    class SwitchExpressionVariants
    {
        string Describe(int n) =>
            n switch
            {
                0 => "zero",
                1 => "one",
                _ => "many",
            };
    }

    // --- NEGATIVE cases: must not be captured -----------------------------------

    class NegativeHolder
    {
        int field;

        void NegativeCases()
        {
            // Lambda binding is not a method_declaration; must never appear as
            // @definition.method / @definition.function.
            System.Action lambdaBinding = () => System.Console.WriteLine("run");

            // Bare field read (no argument_list) must never appear as a call.
            int read = this.field;

            // Field write is not a call.
            this.field = 5;

            // A cast is not a "new"/object-creation reference.
            object boxed = (object)5;
        }

        static string StaticMethod() => "x";
    }
}
