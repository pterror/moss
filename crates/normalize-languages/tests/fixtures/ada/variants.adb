-- Completeness matrix for Ada query files.
-- Each section is commented with the field/variant it exercises, per
-- docs/query-testing-methodology.md step 5. Verified against arborium-ada
-- 2.17.0's node-types.json and real parse output (`normalize syntax ast`
-- / `normalize syntax query --show-source`), not memory.

with Ada.Text_IO;

-- tags.scm: package_declaration.name = identifier (plain package name)
package Plain_Pkg is
   procedure Do_Thing;
end Plain_Pkg;

-- tags.scm: package_declaration.name = selected_component (dotted child
-- package name, e.g. `package Parent.Child is`)
package Plain_Pkg.Child is
   procedure Do_Child_Thing;
end Plain_Pkg.Child;

-- tags.scm: generic_package_declaration wrapping a package_declaration.
-- Must produce exactly ONE @definition.module capture for "Generic_Pkg",
-- not two — a previous version of ada.tags.scm had a dedicated
-- generic_package_declaration pattern that duplicated the plain
-- package_declaration pattern's capture of the same nested node.
generic
   type Elem is private;
package Generic_Pkg is
   procedure Store (X : Elem);
end Generic_Pkg;

package body Variants is

   -- tags.scm: subprogram_declaration/function_specification.name = identifier
   function Plain_Func (X : Integer) return Integer;

   -- tags.scm: subprogram_body/procedure_specification.name = identifier
   procedure Plain_Proc (X : Integer) is
   begin
      null;
   end Plain_Proc;

   -- tags.scm: generic_subprogram_declaration wrapping function_specification
   -- (identifier name). Previously entirely unmatched by ada.tags.scm.
   generic
      type T is private;
   function Generic_Func (X : T) return T;

   -- tags.scm: generic_subprogram_declaration wrapping procedure_specification
   -- (identifier name). Previously entirely unmatched by ada.tags.scm.
   generic
      type T is private;
   procedure Generic_Proc (X : in out T);

   -- types.scm: result_profile.subtype_mark = selected_component
   -- (package-qualified return type)
   function Qualified_Return return Ada.Text_IO.File_Type;

   -- types.scm: component_definition.subtype_mark = selected_component
   -- (package-qualified record field type)
   type Rec_With_Qualified_Field is record
      F : Ada.Text_IO.File_Type;
   end record;

   -- types.scm: subtype_declaration.subtype_mark = selected_component
   subtype Qualified_Subtype is Ada.Text_IO.File_Type;

   -- types.scm: derived_type_definition.subtype_mark = selected_component
   type Qualified_Derived is new Ada.Text_IO.File_Type;

   -- types.scm: object_declaration.subtype_mark = selected_component
   -- (already-handled variant, kept here for completeness-matrix parity)
   Log_File : Ada.Text_IO.File_Type;

   -- types.scm: parameter_specification.subtype_mark = selected_component
   -- (already-handled variant, kept here for completeness-matrix parity)
   procedure Use_File (F : Ada.Text_IO.File_Type);

   -- full_type_declaration: plain (unqualified) identifier child, the
   -- baseline case tags.scm already handled before this sweep.
   type Plain_Type is new Integer;

   -- calls.scm/tags.scm negative case: package-qualified call, must
   -- capture the selector as @call and the prefix as @call.qualifier
   -- (not the whole `Ada.Text_IO.Put_Line` span as one @call).
   procedure Qualified_Call_Demo is
   begin
      Ada.Text_IO.Put_Line ("qualified");
   end Qualified_Call_Demo;

   -- NEGATIVE: bare identifier reference is NOT a call — must not match
   -- (function_call)/(procedure_call_statement).
   procedure Bare_Reference_Demo (Flag : Boolean) is
      Local : Boolean;
   begin
      Local := Flag;
   end Bare_Reference_Demo;

end Variants;
