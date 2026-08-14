-- Completeness matrix for VHDL query files (tags/calls/imports/complexity/types).
-- Each construct is commented with which field/variant it exercises.
-- See docs/query-testing-methodology.md.

-- context declarations/references must precede any other design unit's own
-- library/use clauses in the same file (verified: a preceding `library`/
-- `use` before `context ... is` produces a parse error).
context ieee_ctx is
  library ieee;
  use ieee.std_logic_1164.all;
end context ieee_ctx;

context work.ieee_ctx;                    -- context_reference -> @import.path

library ieee;
use ieee.std_logic_1164.all;              -- use_clause -> @import.path

-- package_declaration.name: identifier
package pkg1 is
  -- full_type_declaration.name: identifier
  type my_int is range 0 to 100;

  -- function_body.designator: operator_symbol (operator overloading)
  function "+" (a, b : integer) return integer;

  -- procedure_body.designator: identifier
  procedure do_thing (a : in integer);
end package pkg1;

package body pkg1 is
  function "+" (a, b : integer) return integer is
  begin
    return a + b;
  end function "+";

  procedure do_thing (a : in integer) is
  begin
    report integer'image(a);              -- attribute_name, designator: predefined_designator + (expression) -> @call
  end procedure do_thing;
end package body pkg1;

-- package_declaration.name: extended_identifier
package \Ext Pkg\ is
  -- full_type_declaration.name: extended_identifier
  type \Ext Type\ is range 0 to 10;

  -- function_body.designator: extended_identifier
  function \My Func\ (a : integer) return integer;

  -- procedure_body.designator: extended_identifier
  procedure \My Proc\ (a : in integer);
end package \Ext Pkg\;

package body \Ext Pkg\ is
  function \My Func\ (a : integer) return integer is
  begin
    return a;
  end function \My Func\;

  procedure \My Proc\ (a : in integer) is
  begin
    report integer'image(a);
  end procedure \My Proc\;
end package body \Ext Pkg\;

use work.pkg1.all;
use work.\Ext Pkg\.all;

-- entity_declaration.name: identifier
entity fifo is
  port (
    clk : in std_logic;                   -- type_mark (simple_name) -> @type.reference
    x   : in \Ext Type\;                  -- type_mark (extended_simple_name) -> @type.reference
    y   : in pkg1.my_int;                 -- type_mark (selected_name suffix: simple_name) -> @type.reference
    z   : in \Ext Pkg\.\Ext Type\;        -- type_mark (selected_name suffix: extended_simple_name)
    s   : in std_logic'subtype             -- type_mark (attribute_name) -> @type.reference
  );
end entity fifo;

-- architecture_body.name: identifier; entity field: simple_name
architecture rtl of fifo is
  signal count : integer := 0;
begin
  -- concurrent conditional signal assignment -> conditional_waveforms @complexity
  count <= 1 when clk = '1' else 0;

  process(clk)
    variable v : integer;
    variable idx : integer := 0;
  begin
    -- ambiguous_name prefix: simple_name -> @call
    v := to_integer(count);

    -- ambiguous_name prefix: extended_simple_name -> @call
    v := \My Func\(1);

    -- ambiguous_name prefix: selected_name (qualified), suffix: simple_name -> @call, @call.qualifier
    v := pkg1.do_thing_result(1);

    -- ambiguous_name prefix: selected_name, suffix: operator_symbol -> @call
    v := pkg1."+"(1, 2);

    -- ambiguous_name prefix: selected_name, suffix: extended_simple_name -> @call
    v := \Ext Pkg\.\My Func\(1);

    -- function_call function: operator_symbol (named-association forces unambiguous form) -> @call
    v := "+"(a => 1, b => 2);

    -- function_call function: extended_simple_name (named-association form) -> @call
    v := \My Func\(a => 1);

    -- procedure_call_statement procedure: identifier -> @call
    do_thing(v);

    -- procedure_call_statement procedure: extended_simple_name -> @call
    \My Proc\(v);

    -- procedure_call_statement procedure: selected_name, suffix: extended_simple_name -> @call, @call.qualifier
    \Ext Pkg\.\My Proc\(v);

    -- if_statement -> @complexity, @nesting
    if clk = '1' then
      idx := idx + 1;
    -- elsif branch inside the same if_statement
    elsif clk = '0' then
      idx := idx - 1;
    else
      idx := 0;
    end if;

    -- case_statement -> @complexity, @nesting
    case idx is
      when 0 =>
        idx := 1;
      when others =>
        idx := 0;
    end case;

    -- loop_statement (for form) -> @complexity, @nesting
    for i in 0 to 3 loop
      exit when i = 2;                    -- exit_statement
      next when i = 1;                    -- next_statement
    end loop;

    -- loop_statement (while form) -> @complexity, @nesting
    while idx < 10 loop
      idx := idx + 1;
    end loop;
  end process;

  -- for_generate_statement -> @nesting
  gen_block : for i in 0 to 3 generate
    -- if_generate_statement -> @complexity, @nesting
    gen_if : if i = 0 generate
      count <= 1;
    elsif i = 1 generate
      count <= 2;
    else generate
      count <= 0;
    end generate gen_if;

    -- case_generate_statement -> @complexity, @nesting
    gen_case : case i generate
      when 0 =>
        count <= 1;
      when others =>
        count <= 0;
    end generate gen_case;
  end generate gen_block;
end architecture rtl;

-- ===========================================================================
-- NEGATIVE cases: constructs that must NOT match as calls/definitions
-- ===========================================================================
architecture negative of fifo is
  signal not_a_call : integer;
begin
  process
    variable v : integer;
  begin
    -- bare signal/variable reference, no parens: must NOT match any @call
    -- pattern (ambiguous_name requires the array-indexing/call shape; a
    -- plain name reference with no argument list is a `simple_name`
    -- directly, never wrapped in `ambiguous_name`/`function_call`).
    v := not_a_call;

    -- attribute reference without arguments must NOT match the attribute
    -- call pattern (no `(expression)` child): rising edge test, not a call.
    if not_a_call'event then
      v := 0;
    end if;
  end process;
end architecture negative;
