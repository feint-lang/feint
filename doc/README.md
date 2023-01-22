# FeInt Documentation

This directory includes additional, non-API documentation for the FeInt
project.

## Building the Documentation

Install `mdbook`:

```shell
cargo install mdbook
```

Then generate the book as HTML:

```shell
mdbook build
```

This will build the documentation into `./build/book`, which can be
deployed as a standalone static site.
