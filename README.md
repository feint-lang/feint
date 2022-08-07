# FeInt

FeInt is a stack-based, bytecode-style interpreter written in Rust.
It's a learning project and is not meant for production use.

## Author

Wyatt Baldwin <code@wyattbaldwin.com>

## License

MIT. See the LICENSE file.

## Ideas

- Everything is an object of some type
- Strong typing
- Lexical scoping
- Almost everything is an expression
- Significant whitespace (by default, but maybe consider `{...}` blocks
  for certain special cases like passing functions)
- No this/self on methods but this/self is required to access attributes
- Disallow arbitrary attachment of attributes (???)
- Everything is immutable by default (???)
- Implicit return of last evaluated expression (like Rust); this applies
  to *all* blocks/scopes
- Custom types can implement operators by defining methods such as `+`
- Only booleans and `nil` can be used in boolean contexts by default;
  custom types can implement the `!!` operator

## Memory Management

TODO

## Builtin Types

- Nil
- Bool (`true` and `false` keywords, not ints)
- Int (BigInt)
- Float (64-bit)
- Str (can use `"` or `'`, multiline)
- Tuple
- List
- BuiltinFunc (e.g., `print()`)
- Func
- Module
- Namespace

## Vars

Variables are defined without the use of any keywords, like Python or
Ruby.

```
a = true
b = nil

a = true
block ->
    print(a)  # outer a
    
a = true
block ->
    a = false
    print(a)  # block a
print(a) # outer a
```

## Format Strings

Similar to f-strings in Python. Sometimes called $-strings since they
use a `$` to mark strings as format strings.

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

block_val == 4
```

## Scopes

In addition to blocks created with `block`, *all* blocks create a new
scope.

Blocks are denoted by `->` and always return (so to speak) a value,
which may be `nil`.

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
if true -> true    # result is true
if false -> false  # result is nil
```

## Match

```
x = "abc"
result = match x ->
    "a"   -> 1
    "ab"  -> 2
    "abc" -> 3
    :     -> 4
print(result)  # -> 3
```

The default branch is denoted by a single `:`. If there's no default
branch and no match is found, the `match` block will return `nil`.

## Loops

```
# Infinite loop
# Use `break` or `break <expression>` to exit
loop ->
    nil

# Loop from 0 up to, but not including, 10
# Expression value is 9 (last value of i)
#
# TODO:
loop i <- 0..10 ->
    i

# Loop from 1 to 10, including 10
# Expression value is 10 (last value of i)
#
# TODO:
loop i <- 1...10 ->
    i

# Loop until condition is met
#
# TODO: There's a bug in this example since `loop cond` checks the OUTER
#       `cond`. This is due to how scopes work--the inner assignment
#       creates a new scope-local var rather than reassigning the outer
#       `cond`. A possible fix would be to add an `outer` keyword.
cond = true
loop cond ->
    cond = false
```

## Jumps

- Forward jumps support the jump-to-exit pattern
- Backward jumps are disallowed (so no looping via goto)
- Labels can't be redefined in a scope
- Can't jump out of functions

```
my_func = (x) ->
    if x ->
        jump exit

    # do stuff and fall through to exit

    exit:
    # clean up and return
```

## Functions

- Lower snake case names only
- Declared/assigned like other vars with `f = () -> ...` syntax
- Value of last evaluated expression is returned
- No explicit `return` is currently possible

```
# Named function
<name> = ([params]) ->
    <block>

<name> = ([params]) -> <expression>

# Anonymous function assigned to a var
<name> = ([params]) ->
    <body>

<name> = ([params]) -> <expression>

# Immediate invocation of anonymous function
(([params]) -> <expression>)([arguments])

my_func = (func) -> func()
my_func(() -> nil)
# -> nil
```

## Custom Types

- Upper camel case names only
- Still working out some details
- Idea: If a method doesn't take any args, allow it to be called with
  or without call syntax?

```
MyType = () =>

    # @ indicates class method
    @new = (value) ->
        this.value = value

    # add operation
    + = (other) ->
        MyType(this.value + other.value)

    # !! must return the bool value of the object
    !! = () ->
        this.value > 10

    # $string must return the string representation of the object
    #
    # NOTE: The $ prefix indicates a special method, similar to
    #       dunder methods in Python (e.g., `__str__`)
    $string = () ->
        $"{this.value}"

obj1 = MyType.new(1)
obj2 = MyType.new(2)
obj1 + obj2
# -> 3
```
