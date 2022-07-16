# FeInt

FeInt is a bytecode interpreter written in Rust. It's a learning project
and is not meant for production use (or any use currently).

## Author

Wyatt Baldwin <code@wyattbaldwin.com>

## License

MIT. See the LICENSE file.

## Ideas

- Everything is an object of some type
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
- Only booleans, numbers, and `nil` can be used in boolean contexts by
  default; custom types can implement the `!!` operator

## Builtin Types

- Nil
- Bool (`true` and `false` keywords, not ints)
- Float (64-bit)
- Int (BigInt)
- String (can use `"` or `'`, multiline)
- Format String (`$"{expr}"`; can use `$"` or `$'`, multiline)
- Tuple
- Option
- Function
- Range (`0..10` and `1...10`)

## Custom Types

- Upper camel case names only

```
MyType () ->

    # @ indicates class method
    @new (value) ->
        this.value = value

    # add operation
    + (other) ->
        MyType(this.value + other.value)

    # $ indicates a special method
    # $bool must return the bool value of the object
    $bool ->
        this.value > 10

    # $string must return the string representation of the object
    $string ->
        $"{this.value}"

obj1 = MyType.new(1)
obj2 = MyType.new(2)
obj1 + obj2
# -> 3
```

## Blocks

Blocks create a new scope and return the value of the last expression.

```
# Expression value is 4
block ->
    x = 2
    y = 2
    x + y
```

## Functions

- Lower snake case names only
- Value of last evaluated expression is returned

```
# Named function
<name> ([parameters]) ->
    <block>

<name> ([parameters]) -> <expression>

# Anonymous function assigned to a var
<name> = ([parameters]) ->
    <body>

<name> = ([parameters]) -> <expression>

# Immediate invocation of anonymous function
(([parameters]) -> <expression>)([arguments])

my_func (func) -> func()
my_func(() -> nil)
# -> nil
```

## Loops

```
# Infinite loop
# Use `break` or `break <expression>` to exit
loop ->
    nil

# Loop from 0 up to, but not including, 10
# Expression value is 9 (last value of i)
loop i <- 0..10 ->
    i

# Loop from 1 to 10, including 10
# Expression value is 10 (last value of i)
loop i <- 1...10 ->
    i

# Loop until condition is met
cond = false
loop cond ->
    cond = true
```

## Jumps

- Forward jumps support the jump-to-exit pattern
- Backward jumps are disallowed (so no looping via goto)
- Labels can't be redefined in a scope

```
my_func (x) ->
    if x ->
        jump exit

    # do stuff and fall through to exit

    exit:
    # clean up and return
```
