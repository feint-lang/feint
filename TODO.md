# TODO

- [-] Fix ident parsing - parse to whitespace and then check for
      validity. Currently, an invalid ident will cause a cryptic
      "expected EOS" error.
- [-] Handle/allow `UPPERCASE_IDENT`s.
- [+] Add `Always` singleton to enable pattern matching.
- [ ] Consider allowing `Map` entries to be accessed using dot notation.
      This would allow `Map`s to be used as lightweight "instances",
      similar to JS.
- [ ] Add docstrings
    - [ ] Functions
    - [ ] Other objects?
- [-] Implement error handling (partially implemented)
- [-] Replace unrecoverable errors with recoverable errors where
      appropriate

- [x] Implement basic types (`Float`, `Int`, `Str`, etc)
- [x] Implement format strings (AKA $ strings)
- [x] Implement `Tuple`
- [x] Implement `List`
- [x] Implement `Map`
- [ ] Implement `Set`
- [x] Implement vars
- [x] Implement dot operator (attribute & item access)
- [x] Implement `block`
- [x] Implement inline blocks (`block -> <expr>`)
- [x] Implement labels
  - [x] Fix `label: <expr>` syntax (this used to work but was broken
    recently)
- [x] Implement `jump` (jump *forward* to label)
  - [x] Allow multiple `jump`s to the same label in a given scope
- [x] Implement conditionals (can be used as ternary too)
- [x] Implement `match`
- [ ] Implement range
- [ ] Implement `loop`
  - [x] Implement while loops
  - [ ] Implement for loops
  - [x] Fix `break` (works for simple cases but is wonky)
  - [x] Implement `continue`
- [x] Implement function calls
  - [ ] Verify implementation
  - [x] Implement `return`
- [x] Implement closures
  - [ ] Verify implementation
- [x] Implement native functions
  - [x] Implement `print` function (`print` is currently implemented as
        a statement and has limited functionality)
- [x] Implement `this` binding for functions
- [x] Check tuple items / args and throw error when invalid items
      are included (e.g., `break` isn't allowed)
- [ ] Implement modules
  - [x] Add `Module` type
  - [ ] Implement module loading
- [ ] Implement `import`
  - [ ] `import <name>`
    - There's a basic version of this that works for builtin modules
  - [ ] `import from <name>: <names>`
- [ ] Implement custom classes
- [ ] Figure out a nice way to do multi-line lambdas
- [-] Improve error handling/reporting (of unrecoverable errors)
  - [ ] Make source location available in AST (started)
  - [ ] Fix locations in format strings
  - [ ] Make source locations available in VM
- [ ] Add a lot more tests
- [-] Profile
- [ ] Benchmark
