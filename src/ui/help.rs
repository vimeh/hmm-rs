use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

// Help section structure
pub struct HelpSection {
    pub title: &'static str,
    pub items: &'static [(&'static str, &'static str)],
}

// Help section definitions
pub const SECTIONS: &[HelpSection] = &[
    HelpSection {
        title: "Navigation:",
        items: &[
            ("h/←", "Move left (parent)"),
            ("j/↓", "Move down"),
            ("k/↑", "Move up"),
            ("l/→", "Move right (child)"),
            ("g  ", "Go to top"),
            ("G  ", "Go to bottom"),
            ("m/~", "Go to root"),
        ],
    },
    HelpSection {
        title: "Editing:",
        items: &[
            ("e/i", "Edit node (append)"),
            ("E/I", "Edit node (replace)"),
            ("o/⏎", "Insert sibling"),
            ("O/⇥", "Insert child"),
            ("d  ", "Delete node"),
            ("D  ", "Delete children"),
        ],
    },
    HelpSection {
        title: "View:",
        items: &[
            ("␣  ", "Toggle collapse"),
            ("v  ", "Collapse all"),
            ("b  ", "Expand all"),
            ("1-5", "Collapse to level"),
        ],
    },
    HelpSection {
        title: "File:",
        items: &[("s  ", "Save"), ("S  ", "Save as"), ("q  ", "Quit")],
    },
];

// Help renderer
pub struct HelpRenderer;

impl HelpRenderer {
    pub fn render(frame: &mut Frame, area: Rect) {
        let help_text = Self::build_help_text();
        let block = Block::default().borders(Borders::ALL).title(" Help ");
        let paragraph = Paragraph::new(help_text)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn build_help_text() -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "h-m-m Help",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];

        for section in SECTIONS {
            lines.push(Line::from(vec![Span::styled(
                section.title,
                Style::default().add_modifier(Modifier::BOLD),
            )]));

            for (key, desc) in section.items {
                lines.push(Line::from(format!("  {}  {}", key, desc)));
            }

            lines.push(Line::from(""));
        }

        lines.push(Line::from("Press ESC or q to close help"));
        lines
    }
}
