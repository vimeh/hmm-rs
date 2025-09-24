use crate::app::{AppMode, AppState};

pub fn start_search(app: &mut AppState) {
    app.mode = AppMode::Search {
        query: String::new(),
    };
}

pub fn type_search_char(app: &mut AppState, c: char) {
    if let AppMode::Search { query } = &mut app.mode {
        query.push(c);
    }
}

pub fn backspace_search(app: &mut AppState) {
    if let AppMode::Search { query } = &mut app.mode {
        query.pop();
    }
}

pub fn confirm_search(app: &mut AppState) {
    if let AppMode::Search { query } = &app.mode {
        // Perform search
        let mut results = Vec::new();
        for node_ref in app.tree.iter() {
            if node_ref
                .get()
                .title
                .to_lowercase()
                .contains(&query.to_lowercase())
            {
                results.push(app.tree.get_node_id(node_ref).unwrap());
            }
        }

        app.search_results = results;
        app.search_index = 0;

        if !app.search_results.is_empty() {
            app.active_node_id = Some(app.search_results[0]);
            app.set_message(format!("Found {} results", app.search_results.len()));
        } else {
            app.set_message("No results found");
        }
    }

    app.mode = AppMode::Normal;
}

pub fn cancel_search(app: &mut AppState) {
    app.mode = AppMode::Normal;
}

pub fn next_search_result(app: &mut AppState) {
    if !app.search_results.is_empty() {
        app.search_index = (app.search_index + 1) % app.search_results.len();
        app.active_node_id = Some(app.search_results[app.search_index]);
        app.set_message(format!(
            "Result {}/{}",
            app.search_index + 1,
            app.search_results.len()
        ));
    }
}

pub fn previous_search_result(app: &mut AppState) {
    if !app.search_results.is_empty() {
        app.search_index = if app.search_index == 0 {
            app.search_results.len() - 1
        } else {
            app.search_index - 1
        };
        app.active_node_id = Some(app.search_results[app.search_index]);
        app.set_message(format!(
            "Result {}/{}",
            app.search_index + 1,
            app.search_results.len()
        ));
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

        // Create a simple tree
        let root = app.tree.new_node(Node::new("Root".to_string()));
        let child1 = app.tree.new_node(Node::new("Child 1".to_string()));
        let child2 = app.tree.new_node(Node::new("Child 2".to_string()));

        root.append(child1, &mut app.tree);
        root.append(child2, &mut app.tree);

        app.root_id = Some(root);
        app.active_node_id = Some(root);

        app
    }

    #[test]
    fn test_search_mode() {
        let mut app = create_test_app();

        start_search(&mut app);
        assert!(matches!(app.mode, AppMode::Search { .. }));

        type_search_char(&mut app, 'C');
        type_search_char(&mut app, 'h');
        type_search_char(&mut app, 'i');

        if let AppMode::Search { query } = &app.mode {
            assert_eq!(query, "Chi");
        }

        confirm_search(&mut app);
        assert!(matches!(app.mode, AppMode::Normal));
        assert!(!app.search_results.is_empty());
    }
}
