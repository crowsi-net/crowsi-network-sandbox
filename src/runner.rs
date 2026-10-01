use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::Path,
    process::{Command, Stdio},
};

const TIMEOUT: &str = "/usr/bin/timeout";
const UNSHARE: &str = "/usr/bin/unshare";
const PINNED_EXECUTABLE: &str = "/proc/self/fd/0";

#[derive(Debug, Clone, Copy)]
pub enum IsolationBudget {
    Short,
    Standard,
}

impl IsolationBudget {
    fn seconds(self) -> &'static str {
        match self {
            Self::Short => "30s",
            Self::Standard => "180s",
        }
    }
}

/// Constructs a finite command with a cleared environment and network namespace.
///
/// # Errors
///
/// Returns an error when the child or required boundary tools are not absolute,
/// regular, non-symlink files.
pub fn isolated_command(
    executable: &Path,
    arguments: &[OsString],
    budget: IsolationBudget,
) -> Result<Command, String> {
    let pinned_executable = pin_executable(executable)?;
    for boundary_tool in [Path::new(TIMEOUT), Path::new(UNSHARE)] {
        validate_boundary_tool(boundary_tool)?;
    }
    let mut command = Command::new(TIMEOUT);
    command
        .stdin(Stdio::from(pinned_executable))
        .env_clear()
        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
        .env("CI", "true")
        .env("NO_COLOR", "1")
        .env("CARGO_NET_OFFLINE", "true")
        .env("NPM_CONFIG_OFFLINE", "true")
        .args(["--kill-after=5s", budget.seconds(), UNSHARE])
        .args(["--user", "--map-current-user", "--net", "--"])
        .arg(PINNED_EXECUTABLE)
        .args(arguments);
    Ok(command)
}

fn pin_executable(path: &Path) -> Result<File, String> {
    validate_absolute(path)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| format!("cannot pin {}: {error}", path.display()))?;
    validate_open_file(path, &file)?;
    Ok(file)
}

fn validate_boundary_tool(path: &Path) -> Result<(), String> {
    let file = pin_executable(path)?;
    if file.metadata().map_err(inspect_error(path))?.uid() == effective_uid()? {
        return Err(format!(
            "boundary tool {} must not be owned by the sandbox identity",
            path.display()
        ));
    }
    Ok(())
}

fn validate_absolute(path: &Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("sandbox executable must use an absolute path".into());
    }
    Ok(())
}

fn validate_open_file(path: &Path, file: &File) -> Result<(), String> {
    let opened = file.metadata().map_err(inspect_error(path))?;
    let current = std::fs::symlink_metadata(path).map_err(inspect_error(path))?;
    let stable = (opened.dev(), opened.ino()) == (current.dev(), current.ino());
    let safe_mode =
        opened.permissions().mode() & 0o022 == 0 && opened.permissions().mode() & 0o111 != 0;
    if !opened.is_file() || current.file_type().is_symlink() || !stable || !safe_mode {
        return Err(format!(
            "sandbox executable {} must be pinned, executable, and not group/world writable",
            path.display()
        ));
    }
    Ok(())
}

fn inspect_error(path: &Path) -> impl FnOnce(std::io::Error) -> String + '_ {
    move |error| format!("cannot inspect {}: {error}", path.display())
}

fn effective_uid() -> Result<u32, String> {
    let status = std::fs::read_to_string("/proc/self/status")
        .map_err(|error| format!("cannot read process identity: {error}"))?;
    status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))
        .and_then(|fields| fields.split_whitespace().nth(1))
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| "cannot determine effective process identity".into())
}
