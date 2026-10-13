# Isolated generated workload

`isolated_command` retains its existing user/network namespace contract. Its inspect schema explicitly reports no filesystem isolation. Use `isolated_workload` when the input must also be separated from owner credentials and host files.

The trusted owner selects a canonical executable, a new owner-only secret-free input directory and finite arguments. The input is bounded to 512 entries, depth eight and 4 MiB; symlinks, hardlinks, sockets, devices and group/world-writable entries are rejected. Do not put credential files, approval objects or owner source/state in the input. The caller controls the host during staging; it must not permit another untrusted host process to rewrite that stage before launch.

The generated `crowsi_process_adapter::Launch` uses `/usr/bin/timeout` and `/usr/bin/bwrap`, verified as executable, non-writable, non-symlink tools not owned by the worker identity. Bubblewrap creates user/PID/network/IPC/UTS/mount namespaces, drops capabilities, starts a new session and clears environment variables. It mounts the reviewed runtime and input read-only, public system `/usr` read-only, isolated `/proc` and `/dev`, and 16 MiB private `/tmp`. Host `/home`, `/etc`, `/run`, owner sockets and other host paths are absent. The root is read-only. This profile requires a supported Linux namespace/bubblewrap environment; an unavailable boundary fails rather than falling back to ordinary execution.

The published `crowsi-process-adapter@0.10.0` owns process start, bounded pipes, parent-death handling, stop and reaping. No new process lifecycle implementation is copied here. The caller must admit bounded stdio messages and expose only the reviewed execute port; never serialize approve functions, credential sources or arbitrary owner commands into that port. A sandbox message is untrusted input and cannot itself authorize a provider operation.

```rust,no_run
use crowsi_network_sandbox::{isolated_workload, IsolationBudget};
use crowsi_process_adapter::OwnedProcess;
use std::{ffi::OsString, path::Path};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let launch = isolated_workload(
    Path::new("/usr/bin/python3.12"), Path::new("/tmp/reviewed-secret-free-input"),
    &[OsString::from("/work/input/worker.py")], IsolationBudget::Short,
)?;
let mut worker = OwnedProcess::spawn(launch).await?;
// Read/admit bounded messages through the owned pipes; keep authority in the host.
worker.shutdown(std::time::Duration::from_millis(100)).await?;
# Ok(()) }
```

Real integration tests use temporary synthetic data and a local fixture listener. They prove that the worker cannot read the synthetic host secret, see the host home or ambient secret, modify input or reach the host listener, and that an uncooperative worker can be stopped/reaped. They do not establish live provider IAM or native credential-daemon integration. Runtime/system libraries are intentionally readable; review the runtime image and public `/usr` content. CPU/memory policy beyond the finite execution timeout belongs to the deployment owner.

Bubblewrap is an external OS prerequisite, not vendored source: [upstream security model](https://github.com/containers/bubblewrap). No live account, persistent credential, production service or timer is used by these tests.

CI pins the hosted Ubuntu 22.04 namespace-capable test environment and builds official bubblewrap 0.13.0 from exact source `719a4fd474d44b26906bcf2b1b0fb6eddd8d56d0` inside that disposable runner. Its old distribution package lacks the required `--size` option. The Ubuntu 24.04 hosted runner rejected bubblewrap loopback setup under its host policy; no host policy is weakened and no direct-execution fallback is enabled. Deployment owners must probe their own namespace policy before admitting workloads.
