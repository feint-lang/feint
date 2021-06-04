# FeInt

FeInt is a bytecode interpreter written in Rust. It's a learning project
and is not meant for production use (or any use currently).

## Author

Wyatt Baldwin <code@wyattbaldwin.com>

## License

MIT. See the LICENSE file.

## Ideas

- Everything is an object of some type
- Avoid keywords
- Lexical scoping
- Everything is an expression
- Significant whitespace (by default, but maybe consider `{...}` blocks
  for certain special cases like passing functions)
- No this/self on methods but this/self is required to access attributes
- Disallow arbitrary attachment of attributes (???)
- Everything is immutable by default (???)

## Builtin Types

- Nil
- Bool (true and false keywords, not ints)
- Float (64-bit)
- Int (BigInt)
- String
- Format String (like f"" in Python)
- Option
- Function
- Range (0..10 and 1...10)

## Types

Upper camel case only

    <Name> ([args])

        # @ indicates class method
        @new (value) ->
            this.value = value

    > Name.new(value)

## Blocks

Blocks create a new scope and return the value of the last expression.

    # Expression value is 4
    block ->
        x = 2
        y = 2
        x + y

## Functions

Lower snake case only. Value of last evaluated expression is returned.

    <name> ([parameters]) ->
        <body>

    <name> = ([parameters]) -> <expression>

    <name> = ([parameters]) ->
        <body>
    
    (([parameters]) -> <expression>)([arguments])

## Loops

    # Loop from 0 up to, but not including, 10
    # Expression value is 9 (last value of i)
    i <- 0..10
        print(i)

    <- true
        break

## Jumps

- Forward jumps support the jump-to-exit pattern
- Backward jumps are disallowed (so no looping via goto)
- Labels can't be redefined in a scope


    my_func () ->
        ...
        if true
            jump exit
        ...
        exit:
        # clean up and return
