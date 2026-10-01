# Security boundary

- Only absolute, executable, non-symlink, non-group/world-writable files are
  accepted. The requested file is pinned before spawn and executed through its
  inherited descriptor, so a later path replacement is not executed.
- The fixed timeout and namespace tools must be outside the sandbox identity's
  ownership. Production packaging must additionally pin their release hashes.
- The child environment is cleared before an allowlisted minimum is added.
- User and network namespaces plus a finite timeout are always requested.
- This package does not claim filesystem, mount, PID, seccomp, or capability
  isolation. Consumers must add those boundaries when required.
- `inspect` and `probe` perform no external network action.

## Private vulnerability reporting

Report vulnerabilities through this repository's GitHub private vulnerability reporting form. Do not put credentials, personal or customer data, or production certificate material in public issues or pull requests.
