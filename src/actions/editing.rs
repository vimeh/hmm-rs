use crate::app::{AppMode, AppState};
use clipboard::{ClipboardContext, ClipboardProvider};

pub fn start_editing(app: &mut AppState, replace: bool) {
    if let Some(active_id) = app.active_node_id {
        let node = app.tree.get(active_id).unwrap().get();
        let buffer = if replace {
            String::new()
        } else {
            node.title.clone()
        };
        let cursor_pos = buffer.len();

        app.mode = AppMode::Editing { buffer, cursor_pos };
    }
}

pub fn type_char(app: &mut AppState, c: char) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        buffer.insert(*cursor_pos, c);
        *cursor_pos += 1;
    }
}

pub fn backspace(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        if *cursor_pos > 0 {
            *cursor_pos -= 1;
            buffer.remove(*cursor_pos);
        }
    }
}

pub fn delete_char(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        if *cursor_pos < buffer.len() {
            buffer.remove(*cursor_pos);
        }
    }
}

pub fn move_cursor_left(app: &mut AppState) {
    if let AppMode::Editing { cursor_pos, .. } = &mut app.mode {
        if *cursor_pos > 0 {
            *cursor_pos -= 1;
        }
    }
}

pub fn move_cursor_right(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        if *cursor_pos < buffer.len() {
            *cursor_pos += 1;
        }
    }
}

pub fn move_cursor_home(app: &mut AppState) {
    if let AppMode::Editing { cursor_pos, .. } = &mut app.mode {
        *cursor_pos = 0;
    }
}

pub fn move_cursor_end(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        *cursor_pos = buffer.len();
    }
}

pub fn move_cursor_word_left(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        if *cursor_pos == 0 {
            return;
        }

        // Move past any spaces
        while *cursor_pos > 0 && buffer.chars().nth(*cursor_pos - 1) == Some(' ') {
            *cursor_pos -= 1;
        }

        // Move to the start of the word
        while *cursor_pos > 0 && buffer.chars().nth(*cursor_pos - 1) != Some(' ') {
            *cursor_pos -= 1;
        }
    }
}

pub fn move_cursor_word_right(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        let len = buffer.len();
        if *cursor_pos >= len {
            return;
        }

        // Move past current word
        while *cursor_pos < len && buffer.chars().nth(*cursor_pos) != Some(' ') {
            *cursor_pos += 1;
        }

        // Move past any spaces
        while *cursor_pos < len && buffer.chars().nth(*cursor_pos) == Some(' ') {
            *cursor_pos += 1;
        }
    }
}

pub fn delete_word_backward(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        if *cursor_pos == 0 {
            return;
        }

        let start_pos = *cursor_pos;

        // Move cursor to start of previous word
        // Skip spaces
        while *cursor_pos > 0 && buffer.chars().nth(*cursor_pos - 1) == Some(' ') {
            *cursor_pos -= 1;
        }

        // Skip word
        while *cursor_pos > 0 && buffer.chars().nth(*cursor_pos - 1) != Some(' ') {
            *cursor_pos -= 1;
        }

        // Delete from cursor to original position
        buffer.replace_range(*cursor_pos..start_pos, "");
    }
}

pub fn delete_word_forward(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        let len = buffer.len();
        if *cursor_pos >= len {
            return;
        }

        let mut end_pos = *cursor_pos;

        // Skip to end of current word
        while end_pos < len && buffer.chars().nth(end_pos) != Some(' ') {
            end_pos += 1;
        }

        // Skip spaces after word
        while end_pos < len && buffer.chars().nth(end_pos) == Some(' ') {
            end_pos += 1;
        }

        // Delete from cursor to end position
        buffer.replace_range(*cursor_pos..end_pos, "");
    }
}

pub fn delete_to_end(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        buffer.truncate(*cursor_pos);
    }
}

pub fn delete_to_start(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        buffer.replace_range(0..*cursor_pos, "");
        *cursor_pos = 0;
    }
}

pub fn paste_at_cursor(app: &mut AppState) {
    if let AppMode::Editing { buffer, cursor_pos } = &mut app.mode {
        // Try to get content from system clipboard
        if let Ok(mut ctx) = ClipboardContext::new() {
            if let Ok(content) = ctx.get_contents() {
                // Clean the content: replace newlines and tabs with spaces
                let cleaned = content
                    .replace('\n', " ")
                    .replace('\r', "")
                    .replace('\t', "  ");

                // Insert at cursor position
                buffer.insert_str(*cursor_pos, &cleaned);
                *cursor_pos += cleaned.len();
            }
        }
    }
}

pub fn confirm_edit(app: &mut AppState) {
    let new_title = if let AppMode::Editing { buffer, .. } = &app.mode {
        buffer.clone()
    } else {
        return;
    };

    if let Some(active_id) = app.active_node_id {
        app.push_history();

        if let Some(node) = app.tree.get_mut(active_id) {
            node.get_mut().title = new_title;
            app.is_dirty = true;
            app.last_modify_time = Some(std::time::Instant::now());
        }
    }
    app.mode = AppMode::Normal;
}

pub fn cancel_edit(app: &mut AppState) {
    app.mode = AppMode::Normal;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::model::Node;

    fn create_test_app() -> AppState {
        let config = AppConfig::default();
        let mut app = AppState::new(config);

        let root = app.tree.new_node(Node::new("Root".to_string()));
        app.root_id = Some(root);
        app.active_node_id = Some(root);

        app
    }

    #[test]
    fn test_edit_mode_type_char() {
        let mut app = create_test_app();
        start_editing(&mut app, true);

        type_char(&mut app, 'T');
        type_char(&mut app, 'e');
        type_char(&mut app, 's');
        type_char(&mut app, 't');

        if let AppMode::Editing { buffer, .. } = &app.mode {
            assert_eq!(buffer, "Test");
        } else {
            panic!("Should be in editing mode");
        }
    }

    #[test]
    fn test_edit_mode_backspace() {
        let mut app = create_test_app();
        start_editing(&mut app, true);

        type_char(&mut app, 'T');
        type_char(&mut app, 'e');
        type_char(&mut app, 's');
        type_char(&mut app, 't');
        backspace(&mut app);

        if let AppMode::Editing { buffer, .. } = &app.mode {
            assert_eq!(buffer, "Tes");
        } else {
            panic!("Should be in editing mode");
        }
    }

    #[test]
    fn test_edit_mode_cursor_movement() {
        let mut app = create_test_app();
        start_editing(&mut app, true);

        type_char(&mut app, 'T');
        type_char(&mut app, 'e');
        type_char(&mut app, 's');
        type_char(&mut app, 't');

        move_cursor_home(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 0);
        }

        move_cursor_end(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 4);
        }

        move_cursor_left(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 3);
        }

        move_cursor_right(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 4);
        }
    }

    #[test]
    fn test_edit_confirm() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        start_editing(&mut app, true);
        type_char(&mut app, 'N');
        type_char(&mut app, 'e');
        type_char(&mut app, 'w');
        confirm_edit(&mut app);

        assert_eq!(app.tree.get(root).unwrap().get().title, "New");
        assert!(matches!(app.mode, AppMode::Normal));
    }

    #[test]
    fn test_edit_cancel() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let original_title = app.tree.get(root).unwrap().get().title.clone();

        start_editing(&mut app, true);
        type_char(&mut app, 'N');
        type_char(&mut app, 'e');
        type_char(&mut app, 'w');
        cancel_edit(&mut app);

        assert_eq!(app.tree.get(root).unwrap().get().title, original_title);
        assert!(matches!(app.mode, AppMode::Normal));
    }

    #[test]
    fn test_cursor_movement() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        app.active_node_id = Some(root);

        // Set up editing mode with test text
        app.mode = AppMode::Editing {
            buffer: "The quick brown fox".to_string(),
            cursor_pos: 19, // At end
        };

        // Test move left
        move_cursor_left(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 18);
        }

        // Test move home
        move_cursor_home(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 0);
        }

        // Test move right
        move_cursor_right(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 1);
        }

        // Test move end
        move_cursor_end(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 19);
        }
    }

    #[test]
    fn test_word_navigation() {
        let mut app = create_test_app();
        app.mode = AppMode::Editing {
            buffer: "The quick brown fox jumps".to_string(),
            cursor_pos: 25, // At end
        };

        // Move word left from end
        move_cursor_word_left(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 20); // Start of "jumps"
        }

        // Move word left again
        move_cursor_word_left(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 16); // Start of "fox"
        }

        // Move word right
        move_cursor_word_right(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 20); // Start of "jumps"
        }

        // Move to beginning and test word right
        move_cursor_home(&mut app);
        move_cursor_word_right(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 4); // Start of "quick"
        }
    }

    #[test]
    fn test_word_deletion() {
        let mut app = create_test_app();

        // Test delete word backward
        app.mode = AppMode::Editing {
            buffer: "The quick brown fox".to_string(),
            cursor_pos: 15, // After "brown"
        };

        delete_word_backward(&mut app);
        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "The quick  fox");
            assert_eq!(*cursor_pos, 10);
        }

        // Test delete word forward
        app.mode = AppMode::Editing {
            buffer: "The quick brown fox".to_string(),
            cursor_pos: 4, // Start of "quick"
        };

        delete_word_forward(&mut app);
        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "The brown fox");
            assert_eq!(*cursor_pos, 4);
        }
    }

    #[test]
    fn test_line_editing() {
        let mut app = create_test_app();

        // Test delete to end
        app.mode = AppMode::Editing {
            buffer: "The quick brown fox".to_string(),
            cursor_pos: 9, // After "quick"
        };

        delete_to_end(&mut app);
        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "The quick");
            assert_eq!(*cursor_pos, 9);
        }

        // Test delete to start
        app.mode = AppMode::Editing {
            buffer: "The quick brown fox".to_string(),
            cursor_pos: 10, // After "quick "
        };

        delete_to_start(&mut app);
        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "brown fox");
            assert_eq!(*cursor_pos, 0);
        }
    }

    #[test]
    fn test_delete_char() {
        let mut app = create_test_app();
        app.mode = AppMode::Editing {
            buffer: "Test".to_string(),
            cursor_pos: 2, // After "Te"
        };

        // Delete character at cursor
        delete_char(&mut app);
        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "Tet");
            assert_eq!(*cursor_pos, 2);
        }

        // Delete at end should do nothing
        move_cursor_end(&mut app);
        delete_char(&mut app);
        if let AppMode::Editing { buffer, .. } = &app.mode {
            assert_eq!(buffer, "Tet");
        }
    }

    #[test]
    fn test_editing_edge_cases() {
        let mut app = create_test_app();

        // Test operations on empty buffer
        app.mode = AppMode::Editing {
            buffer: String::new(),
            cursor_pos: 0,
        };

        // These should not panic
        move_cursor_left(&mut app);
        move_cursor_right(&mut app);
        move_cursor_word_left(&mut app);
        move_cursor_word_right(&mut app);
        delete_word_backward(&mut app);
        delete_word_forward(&mut app);
        backspace(&mut app);
        delete_char(&mut app);

        // Buffer should still be empty
        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "");
            assert_eq!(*cursor_pos, 0);
        }

        // Test word navigation with multiple spaces
        app.mode = AppMode::Editing {
            buffer: "word1   word2".to_string(),
            cursor_pos: 13, // At end
        };

        move_cursor_word_left(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 8); // Start of "word2"
        }

        move_cursor_word_left(&mut app);
        if let AppMode::Editing { cursor_pos, .. } = &app.mode {
            assert_eq!(*cursor_pos, 0); // Start of "word1"
        }
    }

    #[test]
    fn test_start_editing_modes() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        app.active_node_id = Some(root);

        // Test append mode (preserve existing text)
        start_editing(&mut app, false);
        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "Root");
            assert_eq!(*cursor_pos, 4);
        }

        // Reset to normal mode
        app.mode = AppMode::Normal;

        // Test replace mode (clear existing text)
        start_editing(&mut app, true);
        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "");
            assert_eq!(*cursor_pos, 0);
        }
    }

    #[test]
    fn test_editing_basic() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        app.active_node_id = Some(root);

        // Start editing in append mode
        start_editing(&mut app, false);
        assert!(matches!(app.mode, AppMode::Editing { .. }));

        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "Root");
            assert_eq!(*cursor_pos, 4); // At end of "Root"
        }

        // Type some characters
        type_char(&mut app, ' ');
        type_char(&mut app, 'T');
        type_char(&mut app, 'e');
        type_char(&mut app, 's');
        type_char(&mut app, 't');

        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "Root Test");
            assert_eq!(*cursor_pos, 9);
        }

        // Test backspace
        backspace(&mut app);
        if let AppMode::Editing { buffer, cursor_pos } = &app.mode {
            assert_eq!(buffer, "Root Tes");
            assert_eq!(*cursor_pos, 8);
        }

        // Confirm edit
        confirm_edit(&mut app);
        assert!(matches!(app.mode, AppMode::Normal));
        let node = app.tree.get(root).unwrap().get();
        assert_eq!(node.title, "Root Tes");
    }
}
