# Magic Readline Features - Test Guide

## New Editing Features Implemented

### 1. Word Navigation
- **Ctrl+Left/Alt+Left/Alt+B**: Move cursor to previous word
- **Ctrl+Right/Alt+Right/Alt+F**: Move cursor to next word

### 2. Word Deletion
- **Ctrl+Backspace/Alt+Backspace/Ctrl+W**: Delete word before cursor
- **Ctrl+Delete/Alt+D**: Delete word after cursor

### 3. Line Editing
- **Ctrl+K**: Delete from cursor to end of line
- **Ctrl+U**: Delete from start of line to cursor
- **Ctrl+A**: Move to beginning (alternative to Home)
- **Ctrl+E**: Move to end (alternative to End)

### 4. Clipboard Integration
- **Ctrl+V**: Paste from system clipboard at cursor position
  - Newlines are converted to spaces
  - Tabs are converted to double spaces

### 5. Visual Improvements
- **Cursor Indicator**: A visible cursor (▌) shows the current position
- **Scrolling Edit Line**: Long text scrolls to keep cursor visible

## Testing Instructions

1. Run the application: `cargo run test.hmm`
2. Press `e` or `i` to enter edit mode on a node
3. Try the following:
   - Type a long sentence with multiple words
   - Use Ctrl+Arrow to jump between words
   - Use Ctrl+Backspace to delete whole words
   - Use Ctrl+K to delete to end of line
   - Copy some text from another application and use Ctrl+V to paste
   - Watch the cursor indicator (▌) move as you navigate

## Example Test Sequence

1. Edit a node with text: "The quick brown fox jumps over the lazy dog"
2. Press Home (or Ctrl+A) to go to beginning
3. Press Ctrl+Right three times to jump to "fox"
4. Press Ctrl+Backspace to delete "brown"
5. Type "red"
6. Press Ctrl+K to delete everything after "fox"
7. Press Ctrl+V to paste clipboard content

## Implementation Details

All features are implemented following the PHP `magic_readline` function behavior:
- Word boundaries are defined by spaces
- Clipboard content is cleaned (newlines → spaces, tabs → double spaces)
- Cursor position is always visible (text scrolls if needed)
- All standard readline shortcuts are supported