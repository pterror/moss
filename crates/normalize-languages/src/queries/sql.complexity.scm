; Complexity query for SQL
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; SQL complexity comes from CASE expressions (each WHEN branch), MERGE
; statements (each WHEN MATCHED/WHEN NOT MATCHED branch), JOINs, WHERE and
; HAVING clauses that add branching conditions, set operations
; (UNION/INTERSECT/EXCEPT, each a merge point between branches), and EXISTS
; subquery predicates (each a boolean branch condition).
;
; NOTE (bug fix): the original query used `(when_clause) @complexity`,
; believing it covered CASE...WHEN branches. It never did: `when_clause` is
; exclusively the node type for MERGE statement's `WHEN MATCHED`/`WHEN NOT
; MATCHED` clauses (confirmed via real parse — a scalar `CASE WHEN x THEN y
; ELSE z END` expression's `case` node has NO `when_clause` child at all; its
; WHEN branches are flat children: `keyword_when`, condition, `keyword_then`,
; result). This meant CASE expression branches — one of the most common
; branching constructs in real SQL — silently contributed zero complexity in
; every SQL file, ever. `keyword_when` (the literal WHEN token) is common to
; both constructs and correctly counts one branch per WHEN in each.

; Complexity nodes
(keyword_when) @complexity
(join) @complexity
(where) @complexity
(having) @complexity
(set_operation) @complexity
(exists) @complexity

; Nesting nodes
(select) @nesting
(subquery) @nesting
; A CTE (`WITH name AS (...)`) introduces its own nested query scope,
; distinct from `subquery` (which covers subqueries used as expressions).
(cte) @nesting
