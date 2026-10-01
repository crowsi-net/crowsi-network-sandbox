# Crowsi Network Sandbox

Reusable Linux command boundary extracted from Ecosystem Control. It clears the
ambient environment, forces Cargo and npm offline modes, applies a finite
timeout, and runs a command through:

```text
unshare --user --map-current-user --net --
```

`inspect` reports the declared capability without executing a child. `probe`
runs `/usr/bin/true` inside the namespaces to verify host support.
The requested executable is opened with `O_NOFOLLOW`, validated, and inherited
as a pinned descriptor; sandboxed commands intentionally receive no interactive
standard input. The fixed boundary tools must not be owned by the sandbox OS
identity.

```bash
cargo run -- inspect
cargo run -- probe
cargo test
```

This is network isolation, not a container sandbox. It does not create mount,
PID, seccomp, capability, or filesystem boundaries. Incus Projects and
Networks provide the separate server/workload boundary; Crowsi monitors both.
