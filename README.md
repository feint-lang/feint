# FeInt

FeInt is a bytecode interpreter written in Rust. It's a learning project
and is not meant for production use (or any use currently).

## Author

Wyatt Baldwin

## License

MIT. See the LICENSE file.

## Ideas

- Everything is an object of some type
- Avoid keywords
- Everything is immutable by default
- Disallow arbitrary attachment of attributes
- Lexical scoping
- No this/self on methods but this/self is required to access attributes

Builtin Types

- Bool
- Int
- Float
- String
- Char?
- None
- Some
- Option

Types

Upper camel case only

    <Name> ([args])

    @new (value)
        this.value = value

    > Name.new(value)

Functions

Lower snake case only

    <name> ([args]) [-> T]
        <body>

    <name> = ([args]) [-> T] <body>

    <name> = ([args]) [-> T]
        <body>

Loops

      i <- 0..10
          print(i)
