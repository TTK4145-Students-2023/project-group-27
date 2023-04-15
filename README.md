# Elevator Project

This project is designed with a master-slave network configuration with the master and slave nodes
run as separate binaries.

## Master - backup

The master project is located in `master/` and is documented further in it's own [README](master/README.md).

## Slave

The slave project is located in `slave/` and is also documented in it's own [README](slave/README.md).

## Libraries

The folders `network-rust/` and `shared_resources/` contain libraries that are used by both projects.
`network-rust` is a library forked from the handout networking library with some minor tweaks. 
`shared_resources` is a small library containing problem specific structs and methods used by both projects.
