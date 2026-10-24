// SPDX-License-Identifier: MPL-2.0

//! A pane opens a Markdown document by name the way it last showed it.

use super::*;

struct Fixture {
    root: PathBuf,
    notes: PathBuf,
    other: PathBuf,
    app: App,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn fixture(name: &str) -> Fixture {
    let root = temporary(name);
    fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let notes = root.join("notes.md");
    let other = root.join("other.rs");
    fs::write(
        &notes,
        "# Title\n\nSome **bold** text.\n\n## Later\n\nEnd\n",
    )
    .unwrap();
    fs::write(&other, "fn main() {}\n").unwrap();
    let mut app = App::new(Config::default(), Some(notes.clone())).unwrap();
    app.project_root = root.clone();
    app.working_directory = root.clone();
    Fixture {
        root,
        notes,
        other,
        app,
    }
}

fn type_command(app: &mut App, command: &str) {
    press(app, ':');
    for character in command.chars() {
        press(app, character);
    }
    key(app, KeyCode::Enter, Modifiers::NONE);
}

fn shown(app: &App) -> usize {
    app.active().buffer
}

#[test]
fn opening_a_document_by_name_returns_to_the_view_the_pane_left_it_in() {
    let mut f = fixture("page-memory-open");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    assert_ne!(page, source);

    type_command(&mut f.app, &format!("open {}", f.other.display()));
    type_command(&mut f.app, &format!("open {}", f.notes.display()));
    assert_eq!(
        shown(&f.app),
        page,
        "the page was the last view of notes.md"
    );
    assert_eq!(f.app.active().head(), 0);

    press(&mut f.app, '?');
    assert_eq!(shown(&f.app), source);
    type_command(&mut f.app, &format!("open {}", f.other.display()));
    type_command(&mut f.app, &format!("open {}", f.notes.display()));
    assert_eq!(
        shown(&f.app),
        source,
        "the source was the last view of notes.md"
    );
}

#[test]
fn asking_for_the_source_a_pane_already_shows_keeps_it() {
    let mut f = fixture("page-memory-already-shown");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    press(&mut f.app, '?');
    assert_eq!(shown(&f.app), source);

    type_command(&mut f.app, &format!("open {}", f.notes.display()));
    assert_eq!(shown(&f.app), source);
}

#[test]
fn requests_for_a_position_in_the_text_open_the_source() {
    let mut f = fixture("page-memory-positions");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app.open_file(f.other.clone()).unwrap();

    // A content match names a line to edit, as a diagnostic or definition does.
    f.app
        .activate_finder_target(FinderTarget::File(PickerTarget {
            path: f.notes.clone(),
            row: Some(2),
            column: 5,
        }));
    assert_eq!(shown(&f.app), source);
    assert_eq!(cursor(&f.app), Position { row: 2, col: 5 });

    press(&mut f.app, '?');
    assert_eq!(shown(&f.app), page);
    f.app.open_file(f.other.clone()).unwrap();
    f.app.open_file(f.notes.clone()).unwrap();
    assert_eq!(shown(&f.app), source, "open_file is the positional path");
}

#[test]
fn the_finder_opens_an_open_document_by_name_the_way_the_pane_left_it() {
    let mut f = fixture("page-memory-finder");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app.open_document(f.other.clone()).unwrap();

    // An open document is listed by its buffer, and its file row is folded
    // into that one, so this is the row a name search selects.
    f.app.open_project_picker().unwrap();
    for character in "notes.md".chars() {
        press(&mut f.app, character);
    }
    assert_eq!(
        f.app
            .finder
            .as_ref()
            .unwrap()
            .selected_target(f.app.picker.as_ref().unwrap()),
        Some(FinderTarget::Resource(ResourceTarget::Buffer(source)))
    );
    key(&mut f.app, KeyCode::Enter, Modifiers::NONE);
    assert!(f.app.finder.is_none());
    assert_eq!(shown(&f.app), page);

    press(&mut f.app, '?');
    f.app.open_document(f.other.clone()).unwrap();
    f.app
        .activate_finder_target(FinderTarget::Resource(ResourceTarget::Buffer(source)));
    assert_eq!(shown(&f.app), source);
    f.app.open_document(f.other.clone()).unwrap();
    f.app
        .activate_finder_target(FinderTarget::Resource(ResourceTarget::Buffer(page)));
    assert_eq!(shown(&f.app), page, "the page's own row opens the page");
}

#[test]
fn the_source_passed_through_is_not_a_place_to_jump_back_to() {
    let mut f = fixture("page-memory-jumps");
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app.open_document(f.other.clone()).unwrap();
    let other = shown(&f.app);
    f.app.open_document(f.notes.clone()).unwrap();
    assert_eq!(shown(&f.app), page);

    key(&mut f.app, KeyCode::Char('o'), Modifiers::CONTROL);
    assert_eq!(shown(&f.app), other);
    key(&mut f.app, KeyCode::Char('i'), Modifiers::CONTROL);
    assert_eq!(shown(&f.app), page);

    f.app.previous_destination();
    assert_eq!(shown(&f.app), other);
}

#[test]
fn a_heading_link_lands_on_the_heading_of_the_rendered_page() {
    let mut f = fixture("page-memory-heading");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app.open_document(f.other.clone()).unwrap();

    f.app
        .open_file_at_heading(f.notes.clone(), "later")
        .unwrap();
    assert_eq!(shown(&f.app), page);
    assert_eq!(f.app.active().head(), offset_of(&text(&f.app), "Later"));

    // Leaving the page from its heading text returns to that text.
    press(&mut f.app, '?');
    assert_eq!(shown(&f.app), source);
    assert_eq!(f.app.active().head(), offset_of(&text(&f.app), "Later"));

    f.app.open_document(f.other.clone()).unwrap();
    f.app
        .open_file_at_heading(f.notes.clone(), "later")
        .unwrap();
    assert_eq!(shown(&f.app), source);
    assert_eq!(f.app.active().head(), offset_of(&text(&f.app), "## Later"));
}

fn offset_of(text: &str, needle: &str) -> usize {
    text[..text.find(needle).unwrap()].chars().count()
}

#[test]
fn each_pane_remembers_its_own_view_and_a_split_starts_from_its_parent() {
    let mut f = fixture("page-memory-panes");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    let first = f.app.active_pane;

    f.app
        .split_document(Axis::Horizontal, f.other.clone())
        .unwrap();
    let second = f.app.active_pane;
    assert_ne!(first, second);
    f.app.open_document(f.notes.clone()).unwrap();
    assert_eq!(shown(&f.app), page, "the split inherited the page");

    press(&mut f.app, '?');
    assert_eq!(shown(&f.app), source);
    f.app.open_document(f.other.clone()).unwrap();
    f.app.open_document(f.notes.clone()).unwrap();
    assert_eq!(shown(&f.app), source);

    f.app.activate_pane(first);
    f.app.open_document(f.other.clone()).unwrap();
    f.app.open_document(f.notes.clone()).unwrap();
    assert_eq!(
        shown(&f.app),
        page,
        "the first pane still left notes.md rendered"
    );
}

#[test]
fn explorer_entries_and_the_directory_tree_follow_the_view() {
    let mut f = fixture("page-memory-explorer");
    press(&mut f.app, '?');
    let page = shown(&f.app);

    f.app.open_explorer(Some(f.root.clone())).unwrap();
    f.app.focus_directory_entry(&f.notes);
    f.app.open_directory_entry().unwrap();
    assert_eq!(shown(&f.app), page);

    f.app.open_document(f.other.clone()).unwrap();
    f.app.enter_directory_tree();
    f.app.directory_tree.selected = f.notes.clone();
    key(&mut f.app, KeyCode::Enter, Modifiers::NONE);
    assert!(!f.app.directory_tree.focused);
    assert_eq!(shown(&f.app), page);
}

#[test]
fn closing_a_document_closes_its_page_and_closing_the_page_forgets_it() {
    let mut f = fixture("page-memory-close");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app.split(Axis::Horizontal, None).unwrap();
    press(&mut f.app, '?');
    assert_eq!(shown(&f.app), source);

    f.app.close_buffer(source);
    assert!(f.app.closed_buffers.contains(&source));
    assert!(f.app.closed_buffers.contains(&page));
    for pane in f.app.panes.values() {
        assert_ne!(pane.buffer, source);
        assert_ne!(pane.buffer, page);
    }

    f.app.open_document(f.notes.clone()).unwrap();
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app.close_active_buffer(false);
    assert!(f.app.closed_buffers.contains(&page));
    assert!(!f.app.closed_buffers.contains(&source));
    f.app.open_document(f.other.clone()).unwrap();
    f.app.open_document(f.notes.clone()).unwrap();
    assert_eq!(shown(&f.app), source, "closing the page forgot it");
}

#[test]
fn a_page_lives_as_long_as_its_document_despite_special_buffer_retention() {
    let mut f = fixture("page-memory-retention");
    press(&mut f.app, '?');
    let page = shown(&f.app);
    open_filler_special_buffers(&mut f.app, SPECIAL_BUFFER_RETENTION_LIMIT + 2);
    f.app.open_document(f.other.clone()).unwrap();
    f.app.retire_detached_ephemeral_buffers();
    assert!(
        !f.app.closed_buffers.is_empty(),
        "the fillers past the limit were retired"
    );
    assert!(!f.app.closed_buffers.contains(&page));

    f.app.open_document(f.notes.clone()).unwrap();
    assert_eq!(shown(&f.app), page);
}

#[test]
fn closing_hidden_buffers_keeps_a_document_read_through_its_page() {
    let mut f = fixture("page-memory-hidden");
    f.app.open_document(f.other.clone()).unwrap();
    let other = shown(&f.app);
    f.app.open_document(f.notes.clone()).unwrap();
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);

    f.app.open_buffer_picker();
    key(&mut f.app, KeyCode::Tab, Modifiers::NONE);
    let menu = f.app.buffer_action_menu.as_mut().unwrap();
    menu.selected = menu
        .actions
        .iter()
        .position(|action| *action == BufferAction::CloseHidden)
        .unwrap();
    key(&mut f.app, KeyCode::Enter, Modifiers::NONE);
    assert!(f.app.closed_buffers.contains(&other));
    assert!(!f.app.closed_buffers.contains(&source));
    assert!(!f.app.closed_buffers.contains(&page));
    assert_eq!(shown(&f.app), page);
}

#[test]
fn arriving_at_a_page_from_history_or_the_buffer_list_shows_current_text() {
    let mut f = fixture("page-memory-refresh");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    let bold = offset_of(&text(&f.app), "bold");
    f.app.active_mut().replace_selection(Selection::point(bold));
    f.app.open_document(f.other.clone()).unwrap();

    f.app
        .apply_to_buffer(source, &Transaction::insert(0, "Intro paragraph.\n\n"));
    key(&mut f.app, KeyCode::Char('o'), Modifiers::CONTROL);
    assert_eq!(shown(&f.app), page);
    assert!(text(&f.app).starts_with("Intro paragraph."));
    assert_eq!(
        f.app.active().head(),
        offset_of(&text(&f.app), "bold"),
        "the caret follows the text it was on"
    );

    f.app.open_document(f.other.clone()).unwrap();
    let end = f.app.buffers[source].len_chars();
    f.app
        .apply_to_buffer(source, &Transaction::insert(end, "\nAppendix\n"));
    f.app.switch_buffer(page);
    assert!(text(&f.app).contains("Appendix"));
    let revision = f.app.buffers[page].revision();
    f.app.open_document(f.other.clone()).unwrap();
    f.app.switch_buffer(page);
    assert_eq!(
        f.app.buffers[page].revision(),
        revision,
        "an unchanged page is not replaced"
    );
}

#[test]
fn closing_a_buffer_that_falls_back_to_a_page_shows_current_text() {
    let mut f = fixture("page-memory-fallback");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app.open_document(f.other.clone()).unwrap();
    let other = shown(&f.app);
    f.app
        .apply_to_buffer(source, &Transaction::insert(0, "Fresh paragraph.\n\n"));

    f.app.close_buffer(other);
    assert_eq!(shown(&f.app), page);
    assert!(text(&f.app).starts_with("Fresh paragraph."));
}

#[test]
fn returning_to_an_unchanged_page_does_not_render_or_redraw_it() {
    let mut f = fixture("page-memory-unchanged");
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app.open_document(f.other.clone()).unwrap();
    let highlights = f.app.generated_highlights_revision;
    let revision = f.app.buffers[page].revision();
    key(&mut f.app, KeyCode::Char('o'), Modifiers::CONTROL);
    assert_eq!(shown(&f.app), page);
    assert_eq!(f.app.generated_highlights_revision, highlights);
    assert_eq!(f.app.buffers[page].revision(), revision);
}

#[test]
fn closing_hidden_buffers_counts_the_page_that_closes_with_its_document() {
    let mut f = fixture("page-memory-hidden-count");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app.open_document(f.other.clone()).unwrap();

    f.app.open_buffer_picker();
    key(&mut f.app, KeyCode::Tab, Modifiers::NONE);
    let menu = f.app.buffer_action_menu.as_mut().unwrap();
    menu.selected = menu
        .actions
        .iter()
        .position(|action| *action == BufferAction::CloseHidden)
        .unwrap();
    key(&mut f.app, KeyCode::Enter, Modifiers::NONE);
    assert!(f.app.closed_buffers.contains(&source));
    assert!(f.app.closed_buffers.contains(&page));
    assert!(
        f.app.status.contains("closed 2 hidden buffers"),
        "status was {:?}",
        f.app.status
    );
}

#[test]
fn selecting_the_page_already_assigned_to_a_pane_refreshes_it() {
    let mut f = fixture("page-memory-same-page");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    f.app
        .apply_to_buffer(source, &Transaction::insert(0, "Fresh text.\n\n"));
    f.app.switch_buffer(page);
    assert!(text(&f.app).starts_with("Fresh text."));
}

#[test]
fn refreshing_a_shared_page_keeps_both_readers_on_their_text() {
    let mut f = fixture("page-memory-readers");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    let first = f.app.active_pane;
    let bold = offset_of(&text(&f.app), "bold");
    f.app.active_mut().replace_selection(Selection::point(bold));
    f.app.split(Axis::Horizontal, None).unwrap();
    let second = f.app.active_pane;
    let later = offset_of(&text(&f.app), "Later");
    f.app
        .active_mut()
        .replace_selection(Selection::point(later));
    f.app
        .apply_to_buffer(source, &Transaction::insert(0, "Intro.\n\n"));
    f.app.switch_buffer(page);
    let rendered = text(&f.app);
    assert_eq!(f.app.panes[&first].head(), offset_of(&rendered, "bold"));
    assert_eq!(f.app.panes[&second].head(), offset_of(&rendered, "Later"));
}

#[test]
fn retiring_a_document_removes_render_metadata_and_reopening_builds_it_again() {
    let mut f = fixture("page-memory-metadata");
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    assert_eq!(
        f.app.markdown_rendered_from.get(&page),
        Some(&f.app.buffers[source].revision())
    );
    f.app.close_buffer(source);
    assert!(!f.app.markdown_rendered_from.contains_key(&page));
    assert!(!f.app.markdown_positions.contains_key(&page));
    assert!(!f.app.generated_highlights.contains_key(&page));
    f.app.open_document(f.notes.clone()).unwrap();
    let source = shown(&f.app);
    press(&mut f.app, '?');
    let page = shown(&f.app);
    assert_eq!(
        f.app.markdown_rendered_from.get(&page),
        Some(&f.app.buffers[source].revision())
    );
    assert!(f.app.markdown_positions.contains_key(&page));
    assert!(f.app.generated_highlights.contains_key(&page));
}
