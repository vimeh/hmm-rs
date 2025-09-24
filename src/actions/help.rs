use crate::app::{AppMode, AppState};

pub fn show_help(app: &mut AppState) {
    app.mode = AppMode::Help;
}

pub fn close_help(app: &mut AppState) {
    app.mode = AppMode::Normal;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn create_test_app() -> AppState {
        let config = AppConfig::default();
        AppState::new(config)
    }

    #[test]
    fn test_help_mode() {
        let mut app = create_test_app();

        show_help(&mut app);
        assert!(matches!(app.mode, AppMode::Help));

        close_help(&mut app);
        assert!(matches!(app.mode, AppMode::Normal));
    }
}
