// SPDX-License-Identifier: MPL-2.0

//! `:write-config` and `:write-config!` writing the effective configuration.

use super::*;

/// A unique, empty temporary directory per test.
fn scratch(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = temporary_directory().join(format!(
        "runyte-config-write-{}-{nanos}-{name}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

/// An editor started from `source`, with that file as its loaded config.
fn editor(directory: &Path, source: &str) -> App {
    let path = directory.join("loaded.yaml");
    fs::write(&path, source).unwrap();
    let (config, _) = Config::load(Some(&path)).unwrap();
    let mut app = App::new(config, None).unwrap();
    app.note_loaded_config(&path);
    app
}

#[test]
fn write_config_writes_the_whole_effective_configuration_to_a_given_path() {
    let directory = scratch("given");
    let mut app = editor(
        &directory,
        "# a comment the copy does not keep\neditor:\n  tab_width: 2\ntheme: light\nkeys:\n  leader: Ctrl-x\n",
    );
    let target = directory.join("nested").join("written.yaml");

    app.execute_command(&format!("write-config {}", target.display()))
        .unwrap();

    assert!(!app.status_error, "{}", app.status);
    assert!(
        app.status.contains("wrote configuration to"),
        "{}",
        app.status
    );
    let text = fs::read_to_string(&target).unwrap();
    assert!(!text.contains("a comment"), "{text}");
    // Every built-in theme and language server is written, not only the
    // values the loaded file named.
    assert!(
        text.contains(&format!("  {}:\n", crate::config::DEFAULT_THEME)),
        "{text}"
    );
    assert!(text.contains("\nlsp:\n  enable:"), "{text}");
    assert!(!text.contains("servers:"), "{text}");
    let (written, _) = Config::load(Some(&target)).unwrap();
    assert_eq!(written.editor.tab_width, 2);
    assert_eq!(written.theme.as_deref(), Some("light"));
    assert_eq!(
        written.to_yaml().unwrap(),
        app.persisted_config.to_yaml().unwrap()
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn write_config_keeps_an_existing_file_unless_forced() {
    let directory = scratch("existing");
    let mut app = editor(&directory, "editor:\n  tab_width: 3\n");
    let target = directory.join("existing.yaml");
    fs::write(&target, "# hand-written\n").unwrap();

    app.execute_command(&format!("write-config {}", target.display()))
        .unwrap();
    assert!(app.status_error);
    assert!(app.status.contains("already exists"), "{}", app.status);
    assert!(app.status.contains(":write-config!"), "{}", app.status);
    assert_eq!(fs::read_to_string(&target).unwrap(), "# hand-written\n");

    app.execute_command(&format!("write-config! {}", target.display()))
        .unwrap();
    assert!(!app.status_error, "{}", app.status);
    let (written, _) = Config::load(Some(&target)).unwrap();
    assert_eq!(written.editor.tab_width, 3);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn write_config_without_a_path_uses_the_per_user_configuration_file() {
    let directory = scratch("default");
    let mut app = App::new(Config::default(), None).unwrap();
    assert_eq!(app.default_config_target, None);

    app.execute_command("write-config").unwrap();
    assert!(app.status_error);
    assert!(
        app.status.contains("no configuration directory"),
        "{}",
        app.status
    );

    let target = directory.join("runyte").join("config.yaml");
    app.default_config_target = Some(target.clone());
    app.execute_command("write-config").unwrap();
    assert!(!app.status_error, "{}", app.status);
    assert!(Config::load(Some(&target)).is_ok());

    app.execute_command("write-config").unwrap();
    assert!(app.status.contains("already exists"), "{}", app.status);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn write_config_writes_the_saved_value_of_a_setting_awaiting_restart() {
    let directory = scratch("restart");
    let mut app = editor(&directory, "editor:\n  mouse: true\n");
    fs::write(directory.join("loaded.yaml"), "editor:\n  mouse: false\n").unwrap();
    app.execute_command("config-reload").unwrap();
    assert!(app.config.editor.mouse, "the running value is kept");
    let target = directory.join("written.yaml");

    app.execute_command(&format!("write-config {}", target.display()))
        .unwrap();

    let (written, _) = Config::load(Some(&target)).unwrap();
    assert!(!written.editor.mouse);
    fs::remove_dir_all(directory).unwrap();
}
