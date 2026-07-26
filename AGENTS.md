# elk — agent onboarding

A programming language. Functional, Rust-adjacent syntax: ADTs (`type Option<A> { Some(A), None }`),
pattern matching, namespaced functions (`Option::map`), pipes (`|>` / `<|`), partial application,
type inference, compiling via Cranelift.

## Workspace layout

| Crate | Role | State |
|---|---|---|
| `crates/ast` | AST types (`Expression`, `Pattern`, `Type`, `FunctionPath`, …) | healthy |
| `crates/parser` | **winnow 1.0 hand-written parser — the front end** | healthy, 77 tests |
| `crates/inference` | type inference (early; functions keyed by qualified name) | compiles |
| `crates/codegen` | Cranelift backend | **broken: 3 pre-existing cranelift API errors** (`MemFlagsData`, `stack_store` arity, `finalize` signature) — user's WIP, don't "fix" without asking |
| `crates/cli` | binary | — |

## Commands

```bash
cargo test -p parser          # the test suite that matters most right now
cargo clippy                  # must stay at 0 warnings for parser/inference/ast
just lint                     # clippy::nursery — fails only because codegen doesn't compile
just test                     # cargo nextest (needs nextest installed)
```

## Source of truth (deliberately plural — no single one)

History: nom → winnow → (pest experiment) → **winnow, decided on this branch**. Pest is being
sunset. When artifacts disagree, prefer in this order:

1. **Decisions locked in conversation** (see below) — most recent wins.
2. `elk_book/` (symlink to `~/Documents/Elk Book`, Typst) — the most up-to-date prose spec.
   `parts/` is current; `ideas.typ` is stale on syntax but its open-questions list is live.
3. `grammar_optimized.pest` — most up-to-date grammar artifact (updated in lockstep recently).
4. `grammar_revision.pest` > `grammar.pest` (older).
5. `samples/*.elk` — partially stale (still uses `.` for variants).
6. `crates/parser/src/pest_tests.rs` — dead code, not compiled. Sunset material.
7. `book/` — outdated.

## Locked language decisions (this is the law until the user says otherwise)

- **Scoping: `::` for ALL static paths** — variants `Option::Some`, functions `Option::map`,
  `Self::Some`. `.` is exclusively value-level (field access). Rationale: `::` = look inside a
  type/namespace, `.` = look inside a value; makes postfix field chains unambiguous.
- **Generics: angle brackets** — `type Option<A>`, `Option<A>`, `map<A, B>(...)`. Call-site type
  args (`map<A, B>(x, f)`) are valid use cases but DEFERRED (needs inference; also raises the
  Rust-turbofish ambiguity in expression position).
- **Match arms: `=>`** (NOT `->` — that clashes fatally with Pratt infix parsing of `-`).
- **`->`** is reserved for return types and lambdas (`(x) -> x + 1`).
- **Operators: `^` = bitwise xor** (C-convention; Xor is a required low-level op).
  **`**` = exponentiation** — reserved, right-assoc, binds tighter than unary (`-2**2 = -4`);
  NOT implemented yet (no AST variant, no codegen).
- **Precedence = Rust's order** (loosest→tightest): `||` < `&&` < comparisons < `^` < `+-` < `*/%`
  < prefix `-`/`!`. Comparisons are **non-associative** (`a < b < c` stops after the first).
  **NOT** C's order (C puts bitwise looser than `==` — the famous gotcha).
- **Unary: `-` = `Negate` (numeric), `!` = `Not` (boolean)** — never overload `-` onto booleans.
- **Patterns are NOT expressions** — dedicated `ast::patterns::Pattern` type
  (`Wildcard | Identifier | Literal | EnumInstance`). Used in match arms AND fn-impl params
  (pattern-matched multi-impl functions: `Option::unwrap(Option::Some(x)) = x;`).
- **Types are UPPER-case-led** (`TypeIdentifier`), values lower-case-led; `_x` valid, bare `_`
  is the wildcard. Keywords: `main type match return import do` (word-boundaried via `keyword()`).

## Parser architecture (winnow 1.0)

- `parse_expr` is a **Pratt parser** (`winnow::combinator::expression`) with `dispatch!` on the
  operator's first char and binding-power constants at the top of `expressions.rs`.
  `parse_atom` dispatches on the operand's first char.
- The `-` infix branch has a `peek(not('>'))` guard so `->` (lambdas, future) isn't eaten.
- `keyword("...")` combinator in `lib.rs` — ALWAYS use it for keywords (word boundary).
- **Never use `if let Ok(p.parse_next(i))` for optional parses** — failing parsers may have
  consumed input. Use `opt(...)`, which resets on failure. (Bit us in `parse_block_content`.)
- winnow 1.0 gotchas learned the hard way:
  - tuple-`alt` caps at **9 elements** (docs lie about 21); use arrays `[P; N]` (homogeneous) or
    `dispatch!` for more.
  - plain `Result<T>` = `Result<T, ContextError>` always backtracks; `ModalResult`/`ErrMode` is
    opt-in for cut semantics (not adopted yet — error messages are still poor).
  - `try_map` needs `E: std::error::Error`; `&str` isn't — use `verify_map` instead.
  - Never `.unwrap()` a `from_str_radix` etc. in a parser — overflow must fail the parse, not panic.

## Deferred / known gaps (roughly priority order)

1. codegen's 3 cranelift errors (user's WIP).
2. `elk_book` still writes `Self.Some` — needs a `::` sed (offered, not done; it's the user's
   personal doc tree).
3. Postfix chains (`a.b.c`, `f(x).field`) — in the grammar, not in winnow; AST stores calls as
   `name + args`, so this needs AST thought. User was unsure.
4. Cut semantics (`ModalResult` + `cut_err`) for decent error messages.
5. Struct/tuple patterns + tuple/list expressions (AST has no tuple type at all).
6. Blocks as expressions; lambdas `(x) -> expr` (grammar has both, winnow doesn't).
7. `Self` as a real type (currently parses as `Custom("Self", …)`).
8. `TypeVar` constraints (`<S: State>`), imports, comments (`//`, `/* */`) in the winnow parser.
9. Strict string escapes (`\q` currently becomes `q` silently).
10. `BinaryOp::Exp` + `**` implementation when codegen is ready.

## Working style of the user

- Wants to be **asked when in doubt** — pause rather than guess on language-design forks.
- Language-design decisions are made conversationally, then locked. Record new ones in the
  "Locked language decisions" section above when they happen.
- Likes evidence: reproduce suspected bugs with tests before reporting/fixing.
- Careful with `git checkout`/`git restore` — significant work may be uncommitted.
