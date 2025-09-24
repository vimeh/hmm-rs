use crate::app::AppState;

pub fn toggle_symbol(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        if let Some(node) = app.tree.get_mut(active_id) {
            let title = &mut node.get_mut().title;
            let sym1 = format!("{} ", app.config.symbol1);
            let sym2 = format!("{} ", app.config.symbol2);

            if title.starts_with(&sym1) {
                *title = format!("{}{}", sym2, &title[sym1.len()..]);
            } else if title.starts_with(&sym2) {
                *title = title[sym2.len()..].to_string();
            } else {
                *title = format!("{}{}", sym1, title);
            }
        }
    }
}

pub fn sort_siblings(app: &mut AppState) {
    // TODO: Implement sibling sorting
    app.set_message("Sorting not yet implemented");
}

pub fn toggle_numbers(app: &mut AppState) {
    // TODO: Implement numbering
    app.set_message("Numbering not yet implemented");
}

pub fn toggle_hide(app: &mut AppState) {
    if let Some(active_id) = app.active_node_id {
        app.push_history();

        if let Some(node) = app.tree.get_mut(active_id) {
            let title = &mut node.get_mut().title;
            if title.starts_with("[HIDDEN] ") {
                *title = title[9..].to_string();
                app.set_message("Node unhidden");
            } else {
                *title = format!("[HIDDEN] {}", title);
                app.set_message("Node hidden");
            }
        }
    }
}

pub fn toggle_show_hidden(app: &mut AppState) {
    app.config.show_hidden = !app.config.show_hidden;
    app.set_message(format!(
        "Show hidden: {}",
        if app.config.show_hidden { "ON" } else { "OFF" }
    ));
}

pub fn increase_text_width(app: &mut AppState) {
    app.config.max_parent_node_width = (app.config.max_parent_node_width as f32 * 1.2) as usize;
    app.config.max_leaf_node_width = (app.config.max_leaf_node_width as f32 * 1.2) as usize;
    app.set_message(format!(
        "Width: {} / {}",
        app.config.max_parent_node_width, app.config.max_leaf_node_width
    ));
}

pub fn decrease_text_width(app: &mut AppState) {
    app.config.max_parent_node_width =
        ((app.config.max_parent_node_width as f32 / 1.2).max(15.0)) as usize;
    app.config.max_leaf_node_width =
        ((app.config.max_leaf_node_width as f32 / 1.2).max(15.0)) as usize;
    app.set_message(format!(
        "Width: {} / {}",
        app.config.max_parent_node_width, app.config.max_leaf_node_width
    ));
}

pub fn increase_line_spacing(app: &mut AppState) {
    app.config.line_spacing += 1;
    app.set_message(format!("Line spacing: {}", app.config.line_spacing));
}

pub fn decrease_line_spacing(app: &mut AppState) {
    if app.config.line_spacing > 0 {
        app.config.line_spacing -= 1;
    }
    app.set_message(format!("Line spacing: {}", app.config.line_spacing));
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
    fn test_toggle_hide() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();

        toggle_hide(&mut app);
        assert!(app
            .tree
            .get(root)
            .unwrap()
            .get()
            .title
            .starts_with("[HIDDEN] "));

        toggle_hide(&mut app);
        assert!(!app
            .tree
            .get(root)
            .unwrap()
            .get()
            .title
            .starts_with("[HIDDEN] "));
    }

    #[test]
    fn test_toggle_symbol() {
        let mut app = create_test_app();
        let root = app.root_id.unwrap();
        let original_title = app.tree.get(root).unwrap().get().title.clone();

        toggle_symbol(&mut app);
        let title_with_sym1 = app.tree.get(root).unwrap().get().title.clone();
        assert!(title_with_sym1.starts_with(&app.config.symbol1));

        toggle_symbol(&mut app);
        let title_with_sym2 = app.tree.get(root).unwrap().get().title.clone();
        assert!(title_with_sym2.starts_with(&app.config.symbol2));

        toggle_symbol(&mut app);
        let title_without_sym = app.tree.get(root).unwrap().get().title.clone();
        assert_eq!(title_without_sym, original_title);
    }

    #[test]
    fn test_toggle_show_hidden() {
        let mut app = create_test_app();

        let initial_show_hidden = app.config.show_hidden;
        toggle_show_hidden(&mut app);
        assert_ne!(app.config.show_hidden, initial_show_hidden);
    }

    #[test]
    fn test_layout_adjustments() {
        let mut app = create_test_app();
        let initial_width = app.config.max_parent_node_width;
        let initial_spacing = app.config.line_spacing;

        increase_text_width(&mut app);
        assert!(app.config.max_parent_node_width > initial_width);

        decrease_text_width(&mut app);
        assert!(app.config.max_parent_node_width <= initial_width);

        increase_line_spacing(&mut app);
        assert_eq!(app.config.line_spacing, initial_spacing + 1);

        decrease_line_spacing(&mut app);
        assert_eq!(app.config.line_spacing, initial_spacing);
    }
}
