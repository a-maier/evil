# evil

An *ev*ent *il*lustrator for Monte Carlo collider events.

## How to use

You can also find precompiled executables on
[github](https://github.com/a-maier/evil/releases). Start with

    evil EVENTFILE

The event file should be in the LHEF or version 2 of the HepMC
format and can be compressed (bzip2, gzip, lz4, zstd).

If [Rust and Cargo](https://www.rust-lang.org/) are installed on
your system, you can of course compile and run directly from the
source code:

    cargo run --release -- EVENTFILE


License: GPL-3.0-or-later
