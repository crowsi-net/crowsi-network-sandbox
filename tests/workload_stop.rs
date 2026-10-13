use crowsi_network_sandbox::{IsolationBudget, isolated_workload};
use crowsi_process_adapter::OwnedProcess;
use std::{ffi::OsString, fs, os::unix::fs::PermissionsExt, path::Path, time::Duration};
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn trusted_owner_can_stop_and_reap_an_uncooperative_isolated_worker() {
    let dir = tempfile::tempdir().unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        dir.path().join("worker.py"),
        "import time\nprint('ready', flush=True)\nwhile True: time.sleep(1)\n",
    )
    .unwrap();
    let runtime = Path::new("/usr/bin/python3").canonicalize().unwrap();
    let launch = isolated_workload(
        &runtime,
        dir.path(),
        &[OsString::from("/work/input/worker.py")],
        IsolationBudget::Short,
    )
    .unwrap();
    let mut worker = OwnedProcess::spawn(launch).await.unwrap();
    let mut output = worker.take_stdout().unwrap();
    let mut ready = [0; 6];
    tokio::time::timeout(Duration::from_secs(5), output.read_exact(&mut ready))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&ready, b"ready\n");
    let outcome = tokio::time::timeout(
        Duration::from_secs(5),
        worker.shutdown(Duration::from_millis(50)),
    )
    .await
    .unwrap()
    .unwrap();
    assert_ne!(outcome.code(), Some(0));
    assert!(!worker.is_alive().unwrap());
    assert!(worker.pid_for_diagnostics().is_none());
}
