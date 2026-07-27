# The ELK programming language
WIP, not ready for use yet

Inspired by Gleam, Rust, Elm, Haskell and Scala.

## TODO
- Better comment handling
- Expressions evaluate to a type
- Type-bound functions
- Pattern matching
- Lists and tuples
- Standard library
- Type inference
- Tail call optimization
- Mix between rust traits and haskell typeclasses
- String interpolation
- Monads?
- Currying, Piping
- Errors as values, no exceptions
- Cargo-like project manager
- Decide function syntax

# Syntax
## Functions
Examples:
```
// Inline
sum(a: U8, b: U8) -> U8 = a + b;

// Block
sum(a: U8, b: U8) -> U8 {
  a + b
}

// Decoupled, multiple implementations
sum(U8, b: U8) -> U8; // The "labels" are optional when defining the function!
sum(a, 0) = a;
sum(0, b) = b;
sum(a, b) = a + b;

is_origin((U8, U8)) -> Bool;
is_origin((0, 0)) = True;
is_origin(_) = False;
```

### To think about

#### Functions without arguments?

```
my_fn() -> ReturnType;
my_fn -> ReturnType;
```

#### Functions without return type?

```
my_fn(U8) -> Unit;
my_fn(U8);
my_fn x {
  print(x);
}
```

## Matching and custom types
Should all custom types (included those in the stdlib) be fully qualified? ie: `Option.None`

Should all custom types (except those in the stdlib) be fully qualified? ie: Some, `MyType.Var1`

Should the qualification be omitted in scopes where we know the type? Some examples:

```
/// We know the type of `my_val` is `MyType`, so we can omit `MyType.`
match my_val {
  Var1 -> ...
  Var2 -> ...
}
```

Opinion: Maybe having everything fully qualified is better, although it's more verbose.


## Side effects
*Idea:* Pure functions can't call impure functions, but impure functions can call pure functions.
- [ ] Side effect handling? (Monads?, keyword? Something else?)

### Monads
A monad should implement the following functions:
- `Monad.wrap : A -> Monad(A)`
- `Monad.map : Monad(A) -> (A -> B) -> Monad(B)`
- `Monad.flat_map : Monad(A) -> (A -> Monad(B)) -> Monad(B)`
- `Monad.join : Monad(Monad(A)) -> Monad(A)`
- `Monad.unwrap : Monad(A) -> A`
- [ ] Anythig else?

Example:
```
type Option(A) = { None, Some(A) };

Option.wrap(A) -> Option(A);
Option.map(Option(A), f: (A -> B)) -> Option(B);
Option.flat_map(Option(A), f: (A -> Option(B))) -> Option(B);
Option.join(Option(Option(A))) -> Option(A);
Option.unwrap(Option(A)) -> A;
```

- [ ] Monad chaining syntax? like Haskell's `do` notation? Via the `|>` operator? Or using a `chain` function?
