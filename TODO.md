# TODO

- [x] Implement basic types
- [x] Implement format strings (AKA $ strings)
- [x] Implement `Tuple`
- [ ] Implement `List`
- [x] Implement vars
- [x] Implement dot operator (attribute & item access)
- [x] Implement `block`
- [x] Implement inline blocks (`block -> <expr>`)
- [x] Implement labels
  - [x] Fix `label: <expr>` syntax (this used to work but was broken
    recently)
- [x] Implement `jump` (jump *forward* to label)
  - Allow multiple `jump`s to the same label in a given scope
- [x] Implement conditionals
- [ ] Implement ternary operator (can use inline
      `x = if <cond> -> <expr> else -> <expr>` for this so maybe an op
      isn't necessary)
- [ ] Implement `match`?
- [ ] Implement range
- [ ] Implement `loop`
  - [x] Implement while loops
  - [ ] Implement for loops
  - [x] Fix `break` (works for simple cases but is wonky)
  - [x] Implement `continue`
- [x] Implement function calls
  - [ ] Verify implementation
- [x] Implement native functions
  - [x] Implement `print` function (`print` is currently implemented as a statement and has limited functionality)
- [ ] Check tuple items / args and throw error when invalid items are included (e.g., `break` isn't allowed)
- [ ] Implement modules (add `Module` type)
- [ ] Implement custom classes
- [ ] Implement `import`
  - [ ] `import <name>`
  - [ ] `import from <name>: <names>`
- [ ] Figure out a nice way to do multi-line lambdas
- [ ] Improve error handling/reporting
  - [ ] Make source location available in AST (started)
  - [ ] Fix locations in format strings
  - [ ] Make source locations available in VM
- [ ] Add a lot more tests
- [ ] Benchmark
