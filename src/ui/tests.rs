use crate::ui::canvas::BufferCanvas;
use crate::ui::constants::connections;
use crate::ui::text::TextWrapper;

#[test]
fn test_connection_line_constants() {
    // Verify single child connection is 5 dashes
    assert_eq!(
        connections::SINGLE.chars().filter(|&c| c == '─').count(),
        5,
        "Single child connection should have exactly 5 dashes"
    );

    // Verify multi-child connection is 4 dashes
    assert_eq!(
        connections::MULTI.chars().filter(|&c| c == '─').count(),
        4,
        "Multi-child connection should have exactly 4 dashes"
    );

    // Verify hidden variants have the correct dash count
    assert_eq!(
        connections::SINGLE_HIDDEN
            .chars()
            .filter(|&c| c == '─')
            .count(),
        4,
        "Single child connection with hidden should have 4 dashes plus ╫"
    );

    assert_eq!(
        connections::MULTI_HIDDEN
            .chars()
            .filter(|&c| c == '─')
            .count(),
        3,
        "Multi-child connection with hidden should have 3 dashes plus ╫"
    );
}

#[test]
fn test_no_spaces_in_connection_lines() {
    assert!(
        !connections::SINGLE.contains(' '),
        "Single child connection should not contain spaces"
    );
    assert!(
        !connections::MULTI.contains(' '),
        "Multi-child connection should not contain spaces"
    );
    assert!(
        !connections::SINGLE_HIDDEN.contains(' '),
        "Single child connection with hidden should not contain spaces"
    );
    assert!(
        !connections::MULTI_HIDDEN.contains(' '),
        "Multi-child connection with hidden should not contain spaces"
    );
}

#[test]
fn test_buffer_canvas() {
    let mut canvas = BufferCanvas::new(20, 5);

    // Test set_char
    canvas.set_char(5, 2, 'X');
    assert_eq!(canvas.char_buffer[2][5], 'X');

    // Test draw_text
    canvas.draw_text(0, 0, "Hello");
    assert_eq!(&canvas.char_buffer[0][0..5], ['H', 'e', 'l', 'l', 'o']);

    // Test bounds checking
    canvas.set_char(25, 2, 'Y'); // Out of bounds - should not panic
    canvas.set_char(5, 10, 'Z'); // Out of bounds - should not panic

    // Test in_bounds
    assert!(canvas.in_bounds(5, 2));
    assert!(!canvas.in_bounds(20, 2));
    assert!(!canvas.in_bounds(5, 5));
}

#[test]
fn test_text_wrapper() {
    let text = "The quick brown fox jumps over the lazy dog";
    let wrapped = TextWrapper::wrap(text, 10);

    assert!(
        wrapped.len() > 1,
        "Text should be wrapped into multiple lines"
    );
    for line in &wrapped {
        assert!(
            unicode_width::UnicodeWidthStr::width(line.as_str()) <= 10,
            "Line width should not exceed max width"
        );
    }

    // Test empty text
    let empty_wrapped = TextWrapper::wrap("", 10);
    assert_eq!(empty_wrapped.len(), 1);
    assert_eq!(empty_wrapped[0], "");

    // Test single word longer than max width
    let long_word = "verylongword";
    let single_wrapped = TextWrapper::wrap(long_word, 5);
    assert_eq!(single_wrapped.len(), 1);
    assert_eq!(single_wrapped[0], long_word);
}

#[test]
fn test_connection_total_length() {
    use crate::layout::NODE_CONNECTION_SPACING;

    // Total spacing is 6 units
    // With 1 space before connection, we need 5 dashes
    let expected_connection_chars = NODE_CONNECTION_SPACING as usize - 1;

    // Count characters, not bytes
    let actual_chars = connections::SINGLE.chars().count();

    assert_eq!(
        actual_chars, expected_connection_chars,
        "Single child connection should have {} characters to fill spacing",
        expected_connection_chars
    );
}
