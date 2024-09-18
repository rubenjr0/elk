# Function syntax
## 1. Decoupled definition and implementations
- Advantages: Pattern matching, multiple implementations. Arbitrary variable names.
- Disadvantages: Have to remember the order of the types.

Example:
```
is_origin : (U8, U8) -> Bool;
is_origin (0, 0) = True;
is_origin _ = False;
```

## 2. Decoupled definition, single implementation.
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

## 3. Coupled definition and implementation
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
