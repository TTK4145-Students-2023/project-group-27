# Slave

The script responsible for operating a single elevator. 
It sends its own state information and collected requests to the master node, and receives what hall orders to serve from the master node.

## Prerequisites

You will need [the Rust compiler](https://www.rust-lang.org/tools/install) for your operating system. 

If you are testing the program from home, you will have to download [the simulator](https://github.com/TTK4145/Simulator-v2/releases/tag/v1.5) for your specific operating system and copy the executable to this folder.
If running on the lab, all prerequisites should be installed.

## Running the program

Configuration options can be set in a configuration file `config.json` 
by cloning the default `_config.json` file.
If no `config.json` is provided, the default will be used and should work with up to 3 elevators running simultaneously on the same computer provided the command line arguments are used correctly:

The following command line arguments are accepted
* `--elevnum    [num]`: this elevator's number, in the range `0..N_ELEVATORS`. Selects what ports to use, only relevant when running multiple elevators on the same computer. If running more than 3 elevators, ensure you have specified sufficient ports in `config.json`.
* `--serverport [num]`: port for elevator server. Uses default (arduino) port if none is provided. Only relevant if running multiple servers on the same computer

### At the lab (using Arduino elevator)

Because the lab server always uses the same port by default, simply start the elevator server with
```bash
$ elevatorserver
```

Build and run elevator number `elev1`, no need to specify port
```bash
$ cargo run -- --elevnum [elev1]
```

### From home (using elevator simulator)

Running multiple elevators on the same computer requires separate ports for each simulator.
Therefore, start the simulator using
```bash
$ ./SimElevatorServer --port [port1] 
# .\SimElevatorServer.exe --port [port1] on Windows
```

Then, build and run elevator number `elev1` using the same server port
```bash
$ cargo run -- --elevnum [elev1] --serverport [port1]
```
