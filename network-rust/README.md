# Network-rust

A library crate forked from the handout networking library with some minor tweaks.
Notable changes:
- Broadcast senders (`udpnet::bcast::tx`) can be set to only broadcast on `localhost`
- `crossbeam_channel` error are forwareded to the function caller, instead of unwrapped.
