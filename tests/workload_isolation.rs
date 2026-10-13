use crowsi_network_sandbox::{IsolationBudget, isolated_workload};
use crowsi_process_adapter::OwnedProcess;
use std::{
    ffi::OsString,
    fs,
    net::TcpListener,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    time::Duration,
};
use tokio::io::AsyncReadExt;

fn input() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    dir
}

#[tokio::test]
async fn real_worker_cannot_read_host_data_write_inputs_or_reach_host_network() {
    let dir = input();
    let private = input();
    fs::write(private.path().join("secret"), "synthetic-host-secret").unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let script = format!(
        r"
import json, os, socket
from pathlib import Path
result = {{}}
result['secret_visible'] = Path({secret:?}).exists()
result['ambient_secret'] = os.environ.get('FIXTURE_SECRET')
try:
    Path('/work/input/worker.py').write_text('modified')
    result['input_writable'] = True
except OSError:
    result['input_writable'] = False
try:
    socket.create_connection(('127.0.0.1', {port}), timeout=1).close()
    result['host_network'] = True
except OSError:
    result['host_network'] = False
result['host_home_visible'] = Path('/home').exists()
Path('/tmp/result').write_text('ephemeral only')
print(json.dumps(result))
",
        secret = private.path().join("secret").to_string_lossy()
    );
    fs::write(dir.path().join("worker.py"), script).unwrap();
    let runtime = Path::new("/usr/bin/python3").canonicalize().unwrap();
    let launch = isolated_workload(
        &runtime,
        dir.path(),
        &[OsString::from("/work/input/worker.py")],
        IsolationBudget::Short,
    )
    .unwrap()
    .env("FIXTURE_SECRET", "synthetic-ambient-secret");
    let mut worker = OwnedProcess::spawn(launch).await.unwrap();
    let mut output = worker.take_stdout().unwrap().take(4096);
    let mut error = worker.take_stderr().unwrap().take(4096);
    let mut bytes = vec![];
    let mut errors = vec![];
    tokio::time::timeout(Duration::from_secs(10), output.read_to_end(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    error.read_to_end(&mut errors).await.unwrap();
    assert_eq!(
        worker.wait().await.unwrap().code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&errors)
    );
    assert!(bytes.len() < 4096);
    let data: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for field in [
        "secret_visible",
        "input_writable",
        "host_network",
        "host_home_visible",
    ] {
        assert_eq!(data[field], false, "{field}");
    }
    assert!(data["ambient_secret"].is_null());
    assert!(!private.path().join("result").exists());
}

#[test]
fn staging_rejects_symlinks_hardlinks_special_files_and_excessive_arguments() {
    let dir = input();
    let outside = input();
    fs::write(outside.path().join("fixture"), "fixture").unwrap();
    let runtime = Path::new("/usr/bin/python3").canonicalize().unwrap();
    symlink(outside.path().join("fixture"), dir.path().join("link")).unwrap();
    assert!(isolated_workload(&runtime, dir.path(), &[], IsolationBudget::Short).is_err());
    fs::remove_file(dir.path().join("link")).unwrap();
    fs::hard_link(outside.path().join("fixture"), dir.path().join("hardlink")).unwrap();
    assert!(isolated_workload(&runtime, dir.path(), &[], IsolationBudget::Short).is_err());
    fs::remove_file(dir.path().join("hardlink")).unwrap();
    assert!(
        isolated_workload(
            &runtime,
            dir.path(),
            &vec![OsString::from("x"); 65],
            IsolationBudget::Short
        )
        .is_err()
    );
}
