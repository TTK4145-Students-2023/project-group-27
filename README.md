# Elevator Project

This repository contains the code for the elevator project in TTK4145 Real-time programming
for group 27.

## Network topology 

The solution is designed with a master-slave network configuration with the master, backup and slave nodes
run as separate binaries.

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
