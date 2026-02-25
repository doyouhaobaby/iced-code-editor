# TODO

## Bugs

- Ghost cursor and line highlight appears when scrolling or resizing window #8

## Documentation

None

## Features

feat: handle horizontal scrolling when line wrapping is disabled #13

1. **Multiple cursors**

   - Simultaneous editing at multiple positions
   - Requires refactoring cursor from `(usize, usize)` to `Vec<(usize, usize)>`

2. **Code folding**

   - Collapse/expand blocks
   - Indentation-based or syntax-aware

3. **Minimap**

   - Overview of entire file
   - Clickable navigation

4. **Auto-completion**

   - LSP integration
   - Context-aware suggestions

## Performance Improvements

1. **Rope data structure** for better large-file performance
2. **Incremental syntax highlighting** to avoid re-highlighting entire file
3. **Web Worker for highlighting** (when targeting WASM)
