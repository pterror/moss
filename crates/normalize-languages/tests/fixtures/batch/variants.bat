@echo off
REM Completeness matrix for batch.cfg.scm / batch.calls.scm / batch.complexity.scm.
REM The grammar (arborium-batch) is minimal: `goto` and `label`/`:` tokens are
REM unnamed (no fields), `call` is not recognized as a keyword at all, and
REM if/for/while are all collapsed into the generic `keyword` node. See
REM node-types.json and the doc comments in the three .scm files for the
REM verified consequences exercised below.

REM --- genuine label DEFINITION: function_definition at statement start ---
:real_label
echo in real_label

REM --- goto with a colon-prefixed target: emits keyword(goto) then a
REM SEPARATE, SPURIOUS function_definition sibling for the target (known
REM false positive, documented in batch.cfg.scm; not filterable at the
REM query level) ---
goto :real_label

REM --- goto :eof (special "return" target) triggers the same false
REM positive: function_definition(":eof") ---
goto :eof

REM --- call with a colon-prefixed target: `call` is not a recognized
REM keyword (parses as identifier inside an ERROR node), so this produces
REM the same spurious function_definition WITHOUT even a `keyword` sibling
REM to anchor on ---
call :real_label

:second_real_label
echo in second_real_label
exit /b 0
