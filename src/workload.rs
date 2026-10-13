use crate::{IsolationBudget, runner::validate_boundary_tool, workload_input::inspect_input};
use crowsi_process_adapter::Launch;
use std::{
    ffi::OsString,
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

/// Build a Linux workload with read-only input, isolated namespaces and ephemeral scratch.
/// Host credentials, home, account sockets and approval authority are not mounted.
/// The caller owns reviewed runtime/input selection and bounded stdio message admission.
///
/// # Errors
/// Rejects unsafe tools, non-canonical input, special files and oversized launch input.
pub fn isolated_workload(
    runtime: &Path,
    input: &Path,
    arguments: &[OsString],
    budget: IsolationBudget,
) -> Result<Launch, String> {
    for tool in [Path::new("/usr/bin/timeout"), Path::new("/usr/bin/bwrap")] {
        validate_boundary_tool(tool)?;
    }
    let runtime_meta = fs::symlink_metadata(runtime).map_err(|_| "runtime unavailable")?;
    if !runtime.is_absolute()
        || !runtime_meta.is_file()
        || runtime_meta.permissions().mode() & 0o022 != 0
        || runtime_meta.permissions().mode() & 0o111 == 0
        || runtime.canonicalize().map_err(|_| "runtime unavailable")? != runtime
    {
        return Err("reviewed canonical executable required".into());
    }
    let root = fs::symlink_metadata(input).map_err(|_| "input unavailable")?;
    if !input.is_absolute()
        || input.canonicalize().map_err(|_| "input unavailable")? != input
        || !root.is_dir()
        || root.permissions().mode() & 0o077 != 0
    {
        return Err("canonical owner-only input directory required".into());
    }
    let mut count = 0;
    let mut bytes = 0;
    inspect_input(input, root.uid(), &mut count, &mut bytes, 0)?;
    if arguments.len() > 64 || arguments.iter().map(|arg| arg.len()).sum::<usize>() > 32768 {
        return Err("workload arguments exceed bounds".into());
    }
    let mut args: Vec<OsString> = [
        "--kill-after=5s",
        budget.seconds(),
        "/usr/bin/bwrap",
        "--unshare-all",
        "--die-with-parent",
        "--new-session",
        "--clearenv",
        "--cap-drop",
        "ALL",
        "--ro-bind",
        "/usr",
        "/usr",
        "--symlink",
        "usr/bin",
        "/bin",
        "--symlink",
        "usr/lib",
        "/lib",
        "--symlink",
        "usr/lib64",
        "/lib64",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--size",
        "16777216",
        "--tmpfs",
        "/tmp",
        "--dir",
        "/work",
        "--dir",
        "/runtime",
        "--ro-bind",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    args.extend([
        runtime.as_os_str().to_owned(),
        "/runtime/worker".into(),
        "--ro-bind".into(),
        input.as_os_str().to_owned(),
        "/work/input".into(),
    ]);
    args.extend(
        [
            "--remount-ro",
            "/",
            "--chdir",
            "/work",
            "--setenv",
            "HOME",
            "/tmp",
            "--setenv",
            "PATH",
            "/usr/bin:/bin",
            "--",
            "/runtime/worker",
        ]
        .into_iter()
        .map(OsString::from),
    );
    args.extend_from_slice(arguments);
    Ok(Launch::new("/usr/bin/timeout").args(args))
}
