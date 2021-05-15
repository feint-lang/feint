# FeInt Project Template

This is a template for a FeInt project. It shows the basic layout and
includes some source files to give an idea of how things (will
eventually) work.

## Layout

    <package_name>
    ├──README.md
    ├──package.toml
    ├──docs/
    │  ├──index.md
    ├── src/
    │  ├──api.fi    => Exported/public API
    │  ├──main.fi   => main(), like Rust
    │  ├──lib/      => The bulk of the package's code; nothing is
    │  │  ├── x.fi     importable directly from here
    │  │  ├── y.fi
    │  │  ├── z.fi
    ├──tests/
    │  ├──test_x.fi
    │  ├──test_y.fi
    │  ├──test_z.fi

# Exported API

`src/api.fi` defines a package's public API. There is no other way to
access objects from the package. Example:

    export package.x               # exports package.x under <package_name>.x
    export package.x.obj           # exports package.x.obj under <package_name>.x.obj
    export package.x.obj as alias  # exports package.x.obj under <package_name>.alias
    export from package: x
    export from package: x as alias

Within a package, everything in the package is public.

# Imports

Within a package, imports look like:

    import package                   # pulls `package` into namespace
    import package.x                 # pulls `x` into namespace
    import package.x as alias        # pulls `x` into namespace as `alias`
    import from package: x           # pulls `x` into namespace
    import from package: x as alias  # pulls `x` into namespace as `alias`

`package` is reserved word for this purpose in the import context. There
can't be a package named `package`, but the `package` can still be used
as an identifier elsewhere.

From an external package, imports look like this. Basically, it's the
same as above except the actual package name is used instead of the
literal `package`. In addition, only symbols exported via `api.fi` are
accessible. An explicitly defined API is required.

    import <package_name>                    # pulls <package_name> into namespace
    import <package_name>.x                  # pulls `x` into namespace
    import <package_name>.x as alias         # pulls `x` into namespace as `alias`
    import from <package_name>: x            # pulls `x` into namespace
    import from <package_name>: x as alias   # pulls `x` into namespace as `alias`
