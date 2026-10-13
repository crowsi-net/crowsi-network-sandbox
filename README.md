# crowsi-network-sandbox

Run a finite command in a Linux environment with a separate network namespace and bounded resources.

## What you can do

- Use an explicit offline command policy.
- Bound runtime and clean the command environment.
- Compose `isolated_workload` with the published `crowsi-process-adapter` owner: read-only reviewed input, no host home or credentials, isolated PID/network/mount namespaces and 16 MiB ephemeral scratch.

[Workload boundary and example](docs/workload-isolation.md) · [Source contract](docs/oss-source-contract.md)

## Current scope

Linux namespace support and reviewed command configuration are required. This is a command sandbox, not a general deployment service.

Package distribution is not activated by this documentation. Use the checked-in source and the declared dependency versions; published availability must be verified separately.

## Getting started

Install Rust 1.97 or newer and make the declared dependencies available. Third-party and `crowsi-process-adapter` dependencies resolve from public crates.io; no private registry or adjacent checkout is required. Run from this repository:

```sh
cargo test --locked
```

## Documentation and source

[Usage guide](docs/getting-started.md)

[Schemas](schemas) · [Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)
