# Installation

1. Clone the feint repository:

       git@github.com:feint-lang/feint.git

2. Run the installation script:

       make install

   This runs `cargo install` and copies the standard library modules
   into place. Currently, the installation paths are hard coded as
   `~/.cargo/bin/feint` and `~/.local/lib/feint/modules`.
