with Ada.Text_IO; use Ada.Text_IO;
with Ada.Integer_Text_IO;

package body Calculator is

   pragma Inline(Add);

   -- Add two integers and return the result
   function Add(A : Integer; B : Integer) return Integer is
   begin
      return A + B;
   end Add;

   function Subtract(A : Integer; B : Integer) return Integer is
   begin
      return A - B;
   end Subtract;

   function Multiply(A : Integer; B : Integer) return Integer is
      Result : Integer := 0;
   begin
      for I in 1 .. A loop
         Result := Result + B;
      end loop;
      return Result;
   end Multiply;

   function Classify(N : Integer) return String is
   begin
      if N < 0 then
         return "negative";
      elsif N = 0 then
         return "zero";
      else
         return "positive";
      end if;
   end Classify;

   procedure Print_Result(Label : String; Value : Integer) is
   begin
      Put(Label);
      Put(": ");
      Ada.Integer_Text_IO.Put(Value);
      New_Line;
   end Print_Result;

   procedure Run_Demo is
      X : Integer := 10;
      Y : Integer := 3;
   begin
      Print_Result("Add", Add(X, Y));
      Print_Result("Subtract", Subtract(X, Y));
      Print_Result("Multiply", Multiply(X, Y));
      Put_Line(Classify(X));
      Put_Line(Classify(-1));
      Put_Line(Classify(0));
   end Run_Demo;

end Calculator;

-- Child package (dotted name) with a package-qualified return type and
-- generic subprograms — idioms real Ada codebases lean on heavily for
-- library-unit hierarchies and reusable algorithms.
with Ada.Text_IO;

package Calculator.Utils is

   function Get_Log_File return Ada.Text_IO.File_Type;

   generic
      type T is private;
   function Generic_Identity (X : T) return T;

   generic
      type T is private;
   procedure Generic_Swap (X, Y : in out T);

end Calculator.Utils;
