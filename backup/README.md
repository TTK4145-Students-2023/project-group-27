# Backup

This script is responsible for receiving requests from the master node and safely storing them
in case the master program crashes.

## Prerequisites

You will need [the Rust compiler](https://www.rust-lang.org/tools/install) for your operating system.

## Running the program

Configuration options can be set in a configuration file `config.json` by cloning the default `_config.json` file.
If no `config.json` is provided, the default will be used and should work with master, backup and up to 3 elevators all running simultaneously on the same computer.

Build and run using 
```bash
$ cargo run
```
