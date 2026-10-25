// SPDX-License-Identifier: MPL-2.0

use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use crate::config::DEFAULT_THEME;

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

/// A fresh directory holding `config.yaml` and its two fragment directories.
struct Root(PathBuf);

impl Root {
    fn new() -> Self {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "runyte-config-fragments-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn config(&self) -> PathBuf {
        self.0.join("config.yaml")
    }

    fn load(&self) -> Result<Config> {
        Config::load(Some(&self.config())).map(|(config, _)| config)
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn keys(config: &Config) -> String {
    serde_yaml::to_string(config.keys.as_ref().unwrap()).unwrap()
}

#[test]
fn directories_beside_the_file_add_to_it_in_name_order() {
    let root = Root::new();
    root.write(
        "config.yaml",
        "theme: mine\nthemes:\n  mine:\n    accent: '#000001'\nkeys:\n  leader: Ctrl-x\n  bind:\n    normal:\n      F5: save\n      F6: yank\n",
    );
    root.write("themes.d/mine.yaml", "accent: '#000002'\n");
    root.write("themes.d/ocean-dark.yaml", "accent: '#000003'\n");
    root.write("themes.d/.swap.yaml", "not: [valid\n");
    root.write("themes.d/notes.txt", "not a theme\n");
    root.write(
        "keybinds.d/10-first.yaml",
        "leader: Ctrl-a\nbind:\n  normal:\n    F6: undo\n    F7: redo\n",
    );
    root.write(
        "keybinds.d/20-second.yaml",
        "bind:\n  normal:\n    F7: null\n  insert:\n    Ctrl-s: save\n",
    );
    root.write("keybinds.d/30-empty.yaml", "");

    let config = root.load().unwrap();

    assert_eq!(config.themes["mine"].accent, "#000002");
    // A file named after a built-in replaces it, as an inline definition does.
    assert_eq!(config.themes["ocean-dark"].accent, "#000003");
    assert!(config.themes.contains_key(DEFAULT_THEME));
    assert_eq!(
        keys(&config),
        "leader: Ctrl-a\nbind:\n  normal:\n    F5: save\n    F6: undo\n    F7: null\n  insert:\n    Ctrl-s: save\n"
    );
}

#[test]
fn the_directories_load_without_a_configuration_file() {
    let root = Root::new();
    root.write("themes.d/mine.yaml", "accent: '#000002'\n");
    root.write("keybinds.d/normal.yaml", "bind:\n  normal:\n    F5: save\n");

    let config = root.load().unwrap();

    assert_eq!(config.themes["mine"].accent, "#000002");
    assert_eq!(keys(&config), "bind:\n  normal:\n    F5: save\n");
}

#[test]
fn a_broken_file_is_named_in_the_error_and_a_broken_key_shape_is_left_to_the_keymap() {
    let root = Root::new();
    root.write("themes.d/mine.yaml", "accent: [unclosed\n");
    let error = format!("{:#}", root.load().unwrap_err());
    let broken = root.0.join("themes.d").join("mine.yaml");
    assert!(error.contains(&broken.display().to_string()), "{error}");

    let root = Root::new();
    root.write("keybinds.d/normal.yaml", "- not\n- a mapping\n");
    assert_eq!(keys(&root.load().unwrap()), "- not\n- a mapping\n");
}

#[test]
fn files_split_themes_and_keys_out_of_the_configuration_file() {
    let mut config: Config = serde_yaml::from_str(
        "themes:\n  mine:\n    accent: '#000001'\n  Mine:\n    accent: '#000002'\n  'not/a file':\n    accent: '#000003'\nkeys:\n  leader: Ctrl-x\n  bind:\n    normal:\n      F5: save\n    insert:\n      Ctrl-s: save\n    visual:\n      F5: save\n",
    )
    .unwrap();
    config.merge_builtin_defaults();

    let files = config.to_files(Path::new("config.yaml")).unwrap();
    let names = files
        .iter()
        .map(|(path, _)| path.as_path())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "config.yaml",
            "themes.d/Mine.yaml",
            "keybinds.d/00-prefixes.yaml",
            "keybinds.d/normal.yaml",
            "keybinds.d/insert.yaml",
        ]
        .map(Path::new)
    );
    let main = &files[0].1;
    assert!(!main.contains("\nkeys:"), "{main}");
    // Only what a file name cannot carry stays inline: a name that is not a
    // portable file name, and one that differs from another only in case.
    assert!(main.contains("\nthemes:\n  mine:\n"), "{main}");
    assert!(main.contains("\n  not/a file:\n"), "{main}");
    assert!(!main.contains("\n  Mine:\n"), "{main}");
    assert_eq!(
        files[1].1,
        serde_yaml::to_string(&config.themes["Mine"]).unwrap()
    );
    // A mode `bind` does not accept stays with the prefixes, where it is
    // still reported.
    assert_eq!(
        files[2].1,
        "leader: Ctrl-x\nbind:\n  visual:\n    F5: save\n"
    );
    assert_eq!(files[3].1, "bind:\n  normal:\n    F5: save\n");
}

#[test]
fn written_files_load_back_to_the_same_configuration() {
    let root = Root::new();
    let mut config: Config = serde_yaml::from_str(
        "editor:\n  tab_width: 3\ntheme: mine\nthemes:\n  mine:\n    accent: '#000001'\nkeys:\n  rebind:\n    Space g: Leader G\n  bind:\n    normal:\n      F5: save\n      J: null\n",
    )
    .unwrap();
    config.merge_builtin_defaults();
    let files = config.to_files(Path::new("config.yaml")).unwrap();
    for (relative, text) in &files {
        root.write(&relative.to_string_lossy(), text);
    }

    let loaded = root.load().unwrap();

    assert_eq!(loaded.to_files(Path::new("config.yaml")).unwrap(), files);
    assert_eq!(loaded.editor.tab_width, 3);
    assert_eq!(loaded.keys, config.keys);
}

#[test]
fn explicit_null_theme_roles_survive_writing_inline_and_to_a_theme_file() {
    let root = Root::new();
    // Every role whose omission means a default colour rather than null.
    let nulls = [
        "warning",
        "info",
        "change_added",
        "change_modified",
        "change_removed",
        "diff_added",
        "diff_removed",
        "diff_changed",
    ];
    let mut source = String::from("themes:\n  mine:\n    accent: '#000001'\n");
    for role in nulls {
        source.push_str(&format!("    {role}: null\n"));
    }
    let mut config: Config = serde_yaml::from_str(&source).unwrap();
    config.merge_builtin_defaults();
    let theme = &config.themes["mine"];
    assert!(theme.warning.is_none() && theme.diff_added.is_none());
    assert_ne!(*theme, ThemeDefinition::default());

    let inline: Config = serde_yaml::from_str(&config.to_yaml().unwrap()).unwrap();
    assert_eq!(inline.themes["mine"], *theme);

    for (relative, text) in config.to_files(Path::new("config.yaml")).unwrap() {
        root.write(&relative.to_string_lossy(), &text);
    }
    assert_eq!(root.load().unwrap().themes["mine"], *theme);
}
