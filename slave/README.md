# Slave

The script responsible for operating a single elevator. 
It sends its own state information and collected requests to the master node, and receives what hall orders to serve from the master node.

## Running the program

The following command line arguments are accepted
* `--num     [elevnum]`: this elevator's number, in the range `0..N_ELEVATORS`. Selects what ports to use, only relevant when running multiple elevators on the same computer. If running more than 3 elevators, ensure you have specified sufficient ports in `config.json`.
* `--serverport [port]`: port for elevator server. Uses default (arduino) port if none is provided. Only relevant if running multiple servers on the same computer

### At the lab (using Arduino elevator)

Because the lab server always uses the same port by default, simply start the elevator server with
```bash
$ elevatorserver
```

Build and run elevator number `elevnum`, no need to specify server port
```bash
$ cargo run -- --num [elevnum]
```

### From home (using elevator simulator)

Running multiple elevators on the same computer requires separate ports for each simulator.
Therefore, start the simulator using
```bash
$ ./SimElevatorServer --port [port1]
```

Then, build and run elevator number `elevnum` using the same server port
```bash
$ cargo run -- --num [elevnum] --serverport [port1]
```
