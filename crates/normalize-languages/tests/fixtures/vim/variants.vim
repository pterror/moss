" Completeness matrix for VimScript queries (tags/calls/imports).
" One small, commented construct per node-type field variant found by
" cross-referencing vim.{tags,calls,imports}.scm's field constraints
" against arborium-vim's node-types.json (see docs/query-testing-methodology.md).
" Each construct notes which field/variant it exercises. A dedicated
" NEGATIVE section at the bottom holds near-miss constructs that must NOT
" match any of these queries.

" ---------------------------------------------------------------------------
" function_declaration.name variants (vim.tags.scm @definition.function)
" ---------------------------------------------------------------------------

" identifier
function! PlainFunc()
    return 1
endfunction

" scoped_identifier (script-local)
function! s:ScopedFunc()
    return 1
endfunction

" identifier — the `#` separators are part of a plain identifier token,
" not a scoped_identifier (verified via normalize syntax query).
function! foo#bar#AutoloadFunc()
    return 1
endfunction

" field_expression (dict-bound method definition)
let s:obj = {}

function! s:obj.FieldFunc() dict
    return 1
endfunction

" ---------------------------------------------------------------------------
" call_expression.function variants (vim.calls.scm @call)
" ---------------------------------------------------------------------------

" identifier
call PlainFunc()

" scoped_identifier
call s:ScopedFunc()

" field_expression (dict-bound method call)
call s:obj.FieldFunc()

" index_expression (dynamic dispatch-table call)
let s:dispatch = {'go': function('PlainFunc')}
call s:dispatch['go']()

" method_expression `->` chain: NOT its own call_expression.function variant
" in practice — each individual call in the chain (`Filter`, `Join`) still
" parses with a plain `identifier` function field nested inside the
" method_expression wrapper, already covered by the identifier clause above.
" Verified via `normalize syntax query -p <probe> "(call_expression function:
" (method_expression) @f)"` returning 0 matches even on a 3-link chain.
echo [1, 2, 3]->filter({i, v -> v > 1})->join(',')

" ---------------------------------------------------------------------------
" source_statement / runtime_statement variants (vim.imports.scm @import.path)
" ---------------------------------------------------------------------------

" source_statement.file (filename), no bang
source ~/.vim/plain.vim

" source_statement.file (filename), with bang — the bang must NOT leak into
" @import.path (see NEGATIVE section).
source! ~/.vim/optional.vim

" runtime_statement's filenames wrapper, no bang, single path
runtime plugin/single.vim

" runtime_statement's filenames wrapper, with bang and a glob path — the
" bang must NOT leak into @import.path (see NEGATIVE section).
runtime! plugin/*.vim

" ---------------------------------------------------------------------------
" NEGATIVE cases — must not match any of the above queries
" ---------------------------------------------------------------------------

" A bang on its own must never appear as an @import.path capture text.
" (Guarded by the source!/runtime! constructs above; asserted in
" vim_imports_negative_bang_not_captured.)

" A lambda expression is not a function definition and must not appear as
" a @definition.function tag.
let s:Lambda = {x -> x + 1}

" A bare identifier that is not the `function` field of a call_expression
" must not appear as an @call capture.
let s:not_a_call = PlainFunc
