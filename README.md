# The ELK programming language
WIP, not ready for use yet

Inspired by Gleam, Rust, Elm, Haskell and Scala.

## TODO
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

## To think about
### Functions
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

### Side effects
*Idea:* Pure functions can't call impure functions, but impure functions can call pure functions.
- [ ] Side effect handling? (Monads?, Something else?)

### Monads
A monad should implement the following functions:
- `Monad.wrap : A -> Monad(A)`
- `Monad.map : Monad(A) -> (A -> B) -> Monad(B)`
- `Monad.flat_map : Monad(A) -> (A -> Monad(B)) -> Monad(B)`
- `Monad.join : Monad(Monad(A)) -> Monad(A)`
- `Monad.unwrap : Monad(A) -> A`.

Example:
```
type Option(A) = { None | Some(A) };

Option.wrap : A -> Option(A);
Option.map : Option(A) -> (A -> B) -> Option(B);
Option.flat_map : Option(A) -> (A -> Option(B)) -> Option(B);
Option.join : Option(Option(A)) -> Option(A);
Option.unwrap : Option(A) -> A;
```

- [ ] Monad chaining syntax? like Haskell's `do` notation?
