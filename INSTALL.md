# Installation

1. Clone the feint repository:

       git@github.com:feint-lang/feint.git

2. Run the installation script:

       make install

   > NOTE: `make install` is currently just a wrapper around
   > `cargo install --path .`, but it could evolve to handle a more
   > complex installation process.

3. Optionally, install shell completion files:

       make install-bash-completion
       make install-fish-completion
