# Elevator Project

This repository contains the code for the elevator project in TTK4145 Real-time programming
for group 27.

## Design choices

### Network topology

The solution is designed with a master-slave network configuration with the master, backup and slave nodes
run as separate binaries. Each instance also have a process-pair.

The topology is illustrated as such, where the lines represent network connections:
```
§~~~~~~~~~~~~§   §~~~~~~~~~~~~§   §~~~~~~~~~~~~§
§ COMPUTER 0 §   § COMPUTER 1 §   § COMPUTER 2 §
§~~~~~~~~~~~~§   §~~~~~~~~~~~~§   §~~~~~~~~~~~~§
§ backup-----§---§-master-----§---§-slave 2    §
§    ,-------§---§-'    |     §   §            §
§    |       §   §      |     §   §            §
§ slave 0    §   § slave 1    §   §            §
§~~~~~~~~~~~~§   §~~~~~~~~~~~~§   §~~~~~~~~~~~~§
```

### Hall request handling

Hall requests are distributed by the master node, but only after they have been safely stored in the backup.
A request will therefore propegate through the network from the slave node panel were it was requested,
through master and backup and til the slave node it gets distributed to, as shown in the following
diagram

```
slave      master      backup
  |
  | <- request is placed
  |--------->|
             | <- request is picked up by master and is sent to backup for safe storage
             |---------->|
                         | <- request is stored in backup and sent back to master
             |<----------|
             | <- request is distributed by the master to one of the slave
  |<---------|
  | <- the slave controls it's elevator to execute the request
```

At each step, the request is buffered and may be resent a few times in case of packet loss.

## Project structure

The project contains 3 Rust binary crates and running each binary is described in it's README

| Crate | Description |
| --- | --- |
| [`master`](master/README.md) | Binary crate responsible for receiving updates from slaves and distributing orders. |
| [`backup`](backup/README.md) | Binary crate responsible for storing hall orders in case master crashes. |
| [`slave`](slave/README.md) | Binary crate responsible for controlling the elevator and sending updates to master. |

Additionally, the project contains 2 library crates that are used by all binary crates.

| Crate | Desciption |
| --- | --- |
| [`network-rust`](network-rust/README.md) | A library crate forked from the handout networking library with some minor tweaks. |
| [`shared_resources`](shared_resources/README.md) | A small library crate containing problem specific structs and methods used by multiple binary crates. |

## Prerequisites

The software is only stable for Linux environments, but MacOS may also work.
You will need [the Rust compiler](https://www.rust-lang.org/tools/install) for your operating system. 

If you are testing the program from home, you will have to download [the simulator](https://github.com/TTK4145/Simulator-v2/releases/tag/v1.5) for your specific operating system and copy the executable to the `slave/` folder.
If running on the lab, all prerequisites should be installed.

## Configuration

Configuration options can be set in a configuration file `config.json` by cloning the default `_config.json` file.
If no `config.json` is provided, the default will be used and should work with up to 3 elevators running simultaneously on the same computer.
