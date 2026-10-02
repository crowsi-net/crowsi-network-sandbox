# Using crowsi-network-sandbox

Run a finite command in a Linux environment with a separate network namespace and bounded resources.

## Before you start

Linux namespace support and reviewed command configuration are required. This is a command sandbox, not a general deployment service.

## First steps

Run from the repository root:

```sh
cargo test --locked
```

## How to assess the result

- Use an explicit offline command policy.
- Bound runtime and clean the command environment.

A passing source-level check establishes only what that check observes. Keep missing configuration, unavailable services and unverified deployment paths visible.

## Continue reading

[Repository overview](../README.md)
