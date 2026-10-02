# crowsi-network-sandbox

Run a finite command in a Linux environment with a separate network namespace and bounded resources.

## What you can do

- Use an explicit offline command policy.
- Bound runtime and clean the command environment.

## Current scope

Linux namespace support and reviewed command configuration are required. This is a command sandbox, not a general deployment service.

Package distribution is not activated by this documentation. Use the checked-in source and the declared dependency versions; published availability must be verified separately.

## Getting started

Install Rust 1.97 or newer and make the declared dependencies available. Use the configured private registry when a dependency is not distributed publicly. Run from this repository:

```sh
cargo test --locked
```

## Documentation and source

[Usage guide](docs/getting-started.md)

[Schemas](schemas) · [Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)
