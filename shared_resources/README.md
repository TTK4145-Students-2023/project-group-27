# Shared Resources

This crate contains structs shared between the master and the slave nodes and is a
pure library with no binaries.

## Modules

| Module | Description |
| --- | --- |
| `Call` | Data structure representing a request call. That is, what direction the user has requested to travel. |
| `Config` | Data structures and methods for reading a configuration file. |
| `ElevatorMessage` | Data structure representing the data sent from each slave to the master node. |
| `Request` | Data structure representing a requests. That is, what direction and floor is requested. |
| `RequestBuffer` | Data structure buffering requests, temporarily storing requests while waiting from confirmation from other programs. |
