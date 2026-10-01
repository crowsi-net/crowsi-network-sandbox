use crowsi_network_sandbox::{IsolationBudget, isolated_command};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
};

#[test]
fn command_uses_user_and_network_namespaces_with_a_finite_timeout() {
    let command = isolated_command(Path::new("/usr/bin/true"), &[], IsolationBudget::Short)
        .expect("isolated command");
    let arguments = command
        .get_args()
        .map(|item| item.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    for expected in ["30s", "--user", "--map-current-user", "--net", "--"] {
        assert!(arguments.iter().any(|item| item == expected), "{expected}");
    }
    assert!(arguments.iter().any(|item| item == "/proc/self/fd/0"));
    assert!(!arguments.iter().any(|item| item == "/usr/bin/true"));
}

#[test]
fn command_clears_ambient_credentials_and_forces_offline_package_modes() {
    let command = isolated_command(Path::new("/usr/bin/true"), &[], IsolationBudget::Standard)
        .expect("isolated command");
    let environment = command
        .get_envs()
        .map(|(key, value)| {
            (
                key.to_string_lossy().to_string(),
                value.map(|item| item.to_string_lossy().to_string()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        environment.get("CARGO_NET_OFFLINE"),
        Some(&Some("true".into()))
    );
    assert_eq!(
        environment.get("NPM_CONFIG_OFFLINE"),
        Some(&Some("true".into()))
    );
    assert!(!environment.contains_key("GITHUB_TOKEN"));
}

#[test]
fn executable_symlinks_are_rejected_before_spawn() {
    let link = std::env::temp_dir().join(format!("crowsi-link-{}", std::process::id()));
    symlink("/usr/bin/true", &link).unwrap();
    assert!(isolated_command(&link, &[], IsolationBudget::Short).is_err());
    fs::remove_file(link).unwrap();
}

#[test]
fn writable_executables_are_rejected_before_spawn() {
    let path = std::env::temp_dir().join(format!("crowsi-write-{}", std::process::id()));
    fs::write(&path, b"not executed").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(isolated_command(&path, &[], IsolationBudget::Short).is_err());
    fs::remove_file(path).unwrap();
}
