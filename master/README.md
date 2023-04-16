# Master

This script is responsible for receiving messages from the slaves and distributing hall requests among the elevators.

## Prerequisites

You will need [the Rust compiler](https://www.rust-lang.org/tools/install) for your operating system.

## Configuration

Configuration options can be set in a configuration file `config.json` by cloning the default `_config.json` file. If no `config.json` is provided, the default will be used and should work with up to 3 elevators running simultaneously on the same computer. If running more than 3 elevators simultaneously, ensure you have provided sufficient ports in the config file.

The program has support for Windows, Linux and MacOS. If using some other operating system, you will need to compile the [hall requests assigner](https://github.com/TTK4145/Project-resources/tree/master/cost_fns/hall_request_assigner) for your OS and update `config.json` accordingly.

## Running the program

Build and run using 
```bash
$ cargo run
```
