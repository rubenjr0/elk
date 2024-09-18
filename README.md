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
- Decide function syntax

## To think about
### Functions
- [ ] Sntax

#### 1. Decoupled definition and implementations
- Advantages: Pattern matching, multiple implementations. Arbitrary variable names.
- Disadvantages: Have to remember the order of the types.

Example:
```
is_origin : (U8, U8) -> Bool;
is_origin (0, 0) = True;
is_origin _ = False;
```

#### 2. Decoupled definition, single implementation.
Subset of the previous case.
- Advantages: Arbitrary variable names. Simple.
- Disadvantages: No pattern matching, no multiple implementations.

Example:
```
is_origin : (U8, U8) -> Bool;
is_origin x = match x {
  (0, 0) -> True,
  _ -> False
}

// same thing:
is_origin2 : (U8, U8) -> Bool;
is_origin2 x {
  match x {
    (0, 0) -> True,
    _ -> False
  }
}
```

#### 3. Coupled definition and implementation
- Advantages: Don't have to remember the order of the types. Simple.
- Disadvantages: No pattern matching. No multiple implementations. No arbitrary variable names.

Example:
```
// Decide the return type syntax, `:` or `->`?
is_origin(x: (U8, U8)): Bool {
  match x {
    (0, 0) -> True,
    _ -> False
  }
}

square(x: U8): U8 = x * x;
```

---

- [x] Functions without arguments?

```
my_fn : ReturnType;
my_fn = 42;
```

- [x] Functions without return type?

```
my_fn : U8 -> Unit;
my_fn x {
  print(x);
}
```

- [x] Function calling syntax?
> Parentheses should *NOT* be mandatory all the time, just in some cases.

```
my_fn 42 37;
my_fn (other_fn 42) 37;
```

### Matching and custom types
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


### Side effects
*Idea:* Pure functions can't call impure functions, but impure functions can call pure functions.
- [ ] Side effect handling? (Monads?, Something else?)

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
type Option(A) = { None | Some(A) };

Option.wrap : A -> Option(A);
Option.map : Option(A) -> (A -> B) -> Option(B);
Option.flat_map : Option(A) -> (A -> Option(B)) -> Option(B);
Option.join : Option(Option(A)) -> Option(A);
Option.unwrap : Option(A) -> A;
```

- [ ] Monad chaining syntax? like Haskell's `do` notation? Via the `|>` operator? Or using the `chain` function?
