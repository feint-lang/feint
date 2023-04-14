# FeInt

FeInt is a stack-based, bytecode-style interpreter written in Rust.
It's a learning project and a work in progress and is not meant for
production use.

Here's what a simple function definition looks like (more examples
below and in the `examples` directory):

```
raise = (x, p) => x ^ p
```

FeInt is "bytecode-style" in the sense that instructions are defined as
Rust enum variants and not actual byte code. This makes defining and
handling instructions much simpler than in, say, C.

## Author

Wyatt Baldwin <code@wyattbaldwin.com>

## License

MIT. See the LICENSE file.

## Inspiration & Resources

- [Crafting Interpreters](https://craftinginterpreters.com)
- Python, JavaScript, Rust, and a handful of other languages

## Documentation

This README currently contains the most comprehensive documentation.
Work on a dedicated documentation site is in progress:

https://feint-lang.github.io/

## Building & Running

FeInt is a standard Cargo project, so it can built with `cargo build`
and tested with `cargo test`.

TODO: Write a lot more tests.

The REPL can be run with `cargo run` and scripts can be run with
`cargo run <file>`.

NOTE: A script is just a module that may contain a `$main` function.
`$main` is a special name that can only be bound to a function in the
global scope of a module. When a script is run, `$main` will be called
automatically with args passed on the command line (AKA `argv`).

`$main` is equivalent to `if __name__ == "__main__": ...` in Python.

## Writing

There's a work-in-progress [tree-sitter implementation] that includes
instructions for use with [neovim].

[tree-sitter implementation]: https://github.com/wylee/tree-sitter-feint
[neovim]: https://neovim.io/

## Ideas

- Everything is an object of some type
- Strong typing
- Lexical scoping
- Everything is an expression or acts like an expression--every
  statement returns some value
- Significant whitespace (by default, but maybe consider `{...}` blocks
  for certain special cases like passing functions)
- Implicit return of last evaluated expression (like Rust); this applies
  to *all* blocks/scopes
- Only booleans and `nil` can be used in boolean contexts by default;
  custom types can implement the `!!` operator
- Everything is immutable by default (idea: use `=` for `const` and `<-`
  for `var`)
- No this/self in method definitions but this/self is required to access
  attributes
- Disallow arbitrary attachment of attributes
- Custom types can implement operators by defining methods such as `+`
- No inheritance (???)

## Memory Management

The current implementation uses reference counting via Rust's `Arc`, but
memory can leak if there are cycles.

## Intrinsic Types

- Nil
- Bool (`true` and `false` keywords, not ints)
- Int (BigInt)
- Float (64-bit)
- Str (can use `"` or `'`, multiline)
- Tuple
- List
- Map
- Error
- IntrinsicFunc (e.g., `print()`)
- Func
- Module

## Vars

Variables are defined without the use of any keywords, like Python or
Ruby. Vars can be reassigned within a given scope using `=`. Vars in
outer scopes can be reassigned using `<-`.

```
a = true
block ->
    print(a)  # outer a
    
b = true
block ->
    # This creates a new `b` in the block.
    b = false

# After exiting the block, `b` is `true`

c = 1
block ->
    # This reassigns the outer `c`
    c <- 2

# After exiting the block, `c` is `2`
```

## Type Hints

Type hints can be applied to any _identifier_.

NOTE: Currently, type hints are completely ignored and are only useful
      as documentation.

```
a: Int = 1

f: Str = (s: Str, count: Int) =>
    s.repeat(count)
```

## Format Strings

Similar to $-strings in F# and f-strings in Python.

```
x = 1
s = $"{x}"
print(s)
# -> 1
```

## Blocks

Blocks create a new scope and return the value of the last expression.

```
# Expression value is 4
block_val = block ->
    x = 2
    y = 2
    x + y

assert(block_val == 4)
```

## Scopes

In addition to blocks created with `block`, *all* blocks create a new
scope.

Blocks are denoted by `->` and always return (so to speak) a value,
which may be `nil`.

Function blocks are denoted by `=>` to resolve a parsing ambiguity.

## Conditionals

NOTE: By default, only booleans and `nil` can be used in boolean
      contexts. Custom types will be able to define a special property
      name `!!` to allow instances to be used in boolean contexts.

```
# Block style
if true ->
    true
else if 0 > 1 ->
    0
else ->
    false

# Inline style
if true -> true
else if 0 > 1 -> 0
else -> false

# Ternary style
x = if true -> true else -> false

# The else block is optional; nil is returned by default
if true -> true          # result is true
if false -> "1 + 1 = 5"  # result is nil
```

## Match

`match` can be used to simplify `if`/`else if`/`else` blocks where the
same object is always checked. For example, this:

```
x = "abc"
if x == "a" -> 1
else if x == "ab" -> 2
else if x == "abc" -> 3
else -> 4
```

can be written more succinctly as:

```
x = "abc"

result = match x ->
    "a"   -> 1
    "ab"  -> 2
    "abc" -> 3
    *     -> 4

print(result)  # -> 3
```

When a branch matches, its value is returned immediately--there's no
fallthrough.

The default branch is denoted by a single `*`. If there's no default
branch and no match is found, the `match` block will return `nil`.

## Pattern Matching

Pattern matching can be done using the builtin `Always` singleton `@`.
`@` always evaluates as `true`. It can be compared for equality with
_anything_ and the comparison will _always_ be `true`.

```
r = match (1, 2) ->
    (1, @) -> true
    * -> false

assert(r == true)

r = match (1, 2) ->
    (@, 2) -> true
    * -> false

assert(r == true)
```

## Loops

```
# Infinite loop
# Use `break` or `break <expression>` to exit
loop ->
    nil

# Loop from 0 up to, but not including, 10
# Expression value is 9 (last value of i)
#
# TODO: Implement Range & Iterator
loop i <- 0..10 ->
    i

# Loop from 1 to 10, including 10
# Expression value is 10 (last value of i)
#
# TODO: Implement Range & Iterator
loop i <- 1...10 ->
    i

# Loop until condition is met
# 
# NOTE the use of `cond <- false` rather than `cond = false`. This is
# necessary because `cond = false` would create a new var in the loop's
# scope such that the loop would never exit (since it checks the OUTER
# `cond`).
cond = true
loop cond ->
    cond <- false
```

## Jumps

- Forward jumps support the jump-to-exit pattern
- Backward jumps are disallowed (so no looping via goto)
- Labels can only be placed at the beginning of a line or directly
  after a scope start marker (although the latter isn't particularly
  useful)
- Labels are fenced by colons e.g. `:my_label:` (this differentiates
  labels from identifiers with type hints)
- Labels can't be redefined in a scope
- Can't jump out of functions

```
my_func = (x) =>
    if x ->
        jump exit

    # do stuff and fall through to exit

    :exit:
    # clean up and return
```

## Functions

- Lower snake case names only
- Declared/assigned like other vars with `f = () => ...` syntax
- Value of last evaluated expression is returned

```
# Named functions
<name> = ([params]) =>
    <block>

<name> = ([params]) => <expression>

# Immediate invocation of anonymous function
(([params]) => <expression>)([arguments])

my_func = (func) => func()
my_func(() => nil)
# -> nil
```

Functions can be defined with a var args parameter in the last position,
which can be accessed in the body of the function as `$args`. `$args`
is always defined, is always a tuple, and will be empty if there are no
var args.

```
# $main is special; $args is argv
$main = (...) => print($args)

f = (x, y, ...) => print(x, y, $args)
```

### Closures

Closures work pretty much like Python or JavaScript. One difference is
that names defined *after* a function won't be captured.

Here's an extremely artificial example:

```
f = (x) =>
    g = (y) =>
        z = "z"
        h = () =>
            (x, y, z)
            
f1 = f("x1")("y1")
f2 = f("x2")("y2")

r1 = f1()
r2 = f2()

assert(r1 == ("x1", "y1", "z"))
assert(r2 == ("x2", "y2", "z"))
```

NOTE: The implementation hasn't been well-tested and complex closures
      might not work as expected.

## Error Handling

NOTE: Error handling is a major work in progress. There are many
      recoverable errors that are current handled as unrecoverable and
      will cause the interpreter to halt and exit even though they
      shouldn't. This is because there was no real error handling in the
      initial versions.

Unrecoverable runtime errors will cause the interpreter to halt and
exit. These are (or should be--see not above) internal errors for which
no recovery is possible.

Recoverable errors can be "caught" and handled. Currently, they can also
be ignored completely, which isn't optimal.

*Every* object has an `err` attribute, and the result of any expression
may be an error.

The big idea behind this is that all return values can be treated sort
of like `Result<Any, Err>`.

```
assert(!1.err, "1 is not an error")

# Pretend this is some interesting operation that can fail and return an
# error.
result = assert(false)

# Check result - method 1
if result.ok ->
    "ok"
else ->
    ErrType.assertion -> "handle assertion error"
    * -> $"handle other error: {result.err.type}"

# Check result - method 2
if result.err ->
    ErrType.assertion -> "handle assertion error"
    * -> $"handle {result.err.type} error"
else ->
    "ok"

# Check result - method 3
match result.err.type ->
    ErrType.ok -> "ok"
    ErrType.assertion -> "handle assertion error"
    * -> $"handle other error: {result.err.type}"
```

## Custom Types

TODO: Custom types are still in the idea phase and haven't been 
      implemented.

- Upper camel case names only
- Still working out some details
- Idea: If a method doesn't take any args, allow it to be called with
  or without call syntax?

```
MyType = () =>

    # @ indicates class method
    @new = (value) =>
        this.value = value

    # add operation
    + = (other) =>
        MyType(this.value + other.value)

    # !! must return the bool value of the object
    !! = () =>
        this.value > 10

    # $string must return the string representation of the object
    #
    # NOTE: The $ prefix indicates a special method, similar to
    #       dunder methods in Python (e.g., `__str__`)
    $string = () =>
        $"{this.value}"

obj1 = MyType.new(1)
obj2 = MyType.new(2)
obj1 + obj2
# -> 3
```
