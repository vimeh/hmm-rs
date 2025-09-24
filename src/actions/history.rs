use crate::app::AppState;

pub fn undo(app: &mut AppState) {
    if app.undo() {
        app.set_message("Undone");
    } else {
        app.set_message("Nothing to undo");
    }
}

pub fn redo(app: &mut AppState) {
    if app.redo() {
        app.set_message("Redone");
    } else {
        app.set_message("Nothing to redo");
    }
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
    fn test_undo_redo() {
        let mut app = create_test_app();

        // The initial tree should have "Root" as the title
        let initial_title = app
            .tree
            .get(app.root_id.unwrap())
            .unwrap()
            .get()
            .title
            .clone();
        assert_eq!(initial_title, "Root");

        // Save initial state to history
        app.push_history();

        // Make a change and save it
        let root = app.root_id.unwrap();
        app.tree.get_mut(root).unwrap().get_mut().title = "Modified".to_string();
        app.push_history();

        // Make another change (current state, not in history yet)
        app.tree.get_mut(root).unwrap().get_mut().title = "Modified2".to_string();

        // Verify we have the current state
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Modified2"
        );

        // Undo - should go back to "Modified" (the last saved state)
        undo(&mut app);
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Modified"
        );

        // Undo again - should go back to "Root"
        undo(&mut app);
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Root"
        );

        // Redo - should go forward to "Modified"
        redo(&mut app);
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Modified"
        );

        // Redo again - should not change since we lost "Modified2" when we did undo
        redo(&mut app);
        assert!(app.message.is_some()); // Should have "Nothing to redo" message
        assert_eq!(
            app.tree.get(app.root_id.unwrap()).unwrap().get().title,
            "Modified"
        );
    }
}
