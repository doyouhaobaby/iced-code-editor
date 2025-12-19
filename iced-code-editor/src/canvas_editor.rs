//! Canvas-based text editor widget for maximum performance.
//!
//! This module provides a custom Canvas widget that handles all text rendering
//! and input directly, bypassing Iced's higher-level widgets for optimal speed.

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry};
use iced::widget::operation::scroll_to;
use iced::widget::{Id, Scrollable, container, scrollable};
use iced::{
    Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Task, Theme, keyboard,
};
use std::time::{Duration, Instant};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;

use crate::state::EditorTheme;
use crate::text_buffer::TextBuffer;

use iced::widget::canvas::Action;

/// Canvas-based text editor constants
const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 20.0;
const CHAR_WIDTH: f32 = 8.4; // Monospace character width
const GUTTER_WIDTH: f32 = 60.0;
const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(530);

/// Canvas-based high-performance text editor.
#[allow(dead_code)] // Some fields will be used when features are complete
pub struct CanvasEditor {
    /// Text buffer
    buffer: TextBuffer,
    /// Cursor position (line, column)
    cursor: (usize, usize),
    /// Scroll offset in pixels
    scroll_offset: f32,
    /// Editor theme
    theme: EditorTheme,
    /// Syntax highlighting language
    syntax: String,
    /// Last cursor blink time
    last_blink: Instant,
    /// Cursor visible state
    cursor_visible: bool,
    /// Selection start (if any)
    selection_start: Option<(usize, usize)>,
    /// Selection end (if any) - cursor position during selection
    selection_end: Option<(usize, usize)>,
    /// Mouse is currently dragging for selection
    is_dragging: bool,
    /// Cache for canvas rendering
    cache: canvas::Cache,
    /// Scrollable ID for programmatic scrolling
    scrollable_id: Id,
    /// Current viewport scroll position (Y offset)
    viewport_scroll: f32,
    /// Viewport height (visible area)
    viewport_height: f32,
}

/// Messages emitted by the canvas editor
#[derive(Debug, Clone)]
pub enum CanvasEditorMessage {
    /// Character typed
    CharacterInput(char),
    /// Backspace pressed
    Backspace,
    /// Delete pressed
    Delete,
    /// Enter pressed
    Enter,
    /// Arrow key pressed (direction, shift_pressed)
    ArrowKey(ArrowDirection, bool),
    /// Mouse clicked at position
    MouseClick(Point),
    /// Mouse drag for selection
    MouseDrag(Point),
    /// Mouse released
    MouseRelease,
    /// Copy selected text (Ctrl+C)
    Copy,
    /// Paste text from clipboard (Ctrl+V)
    Paste(String),
    /// Request redraw for cursor blink
    Tick,
    /// Page Up pressed
    PageUp,
    /// Page Down pressed
    PageDown,
    /// Home key pressed (move to start of line, shift_pressed)
    Home(bool),
    /// End key pressed (move to end of line, shift_pressed)
    End(bool),
    /// Viewport scrolled - track scroll position
    Scrolled(scrollable::Viewport),
}

/// Arrow key directions
#[derive(Debug, Clone, Copy)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

impl CanvasEditor {
    /// Creates a new canvas-based editor.
    ///
    /// # Arguments
    ///
    /// * `content` - Initial text content
    /// * `syntax` - Syntax highlighting language
    ///
    /// # Returns
    ///
    /// A new `CanvasEditor` instance
    #[must_use]
    pub fn new(content: &str, syntax: &str) -> Self {
        Self {
            buffer: TextBuffer::new(content),
            cursor: (0, 0),
            scroll_offset: 0.0,
            theme: EditorTheme::dark(),
            syntax: syntax.to_string(),
            last_blink: Instant::now(),
            cursor_visible: true,
            selection_start: None,
            selection_end: None,
            is_dragging: false,
            cache: canvas::Cache::default(),
            scrollable_id: Id::unique(),
            viewport_scroll: 0.0,
            viewport_height: 600.0, // Default, will be updated
        }
    }

    /// Updates the editor state based on messages and returns scroll commands.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// A Task that may contain scroll commands to keep cursor visible
    pub fn update(&mut self, message: &CanvasEditorMessage) -> Task<CanvasEditorMessage> {
        match message {
            CanvasEditorMessage::CharacterInput(ch) => {
                let (line, col) = self.cursor;
                self.buffer.insert_char(line, col, *ch);
                self.cursor.1 += 1;
                self.reset_cursor_blink();
                self.cache.clear();
                Task::none()
            }
            CanvasEditorMessage::Backspace => {
                let (line, col) = self.cursor;
                if self.buffer.delete_char(line, col) {
                    // Line merged with previous
                    if line > 0 {
                        let prev_line_len = self.buffer.line_len(line - 1);
                        self.cursor = (line - 1, prev_line_len);
                    }
                } else if col > 0 {
                    self.cursor.1 -= 1;
                }
                self.reset_cursor_blink();
                self.cache.clear();
                self.scroll_to_cursor()
            }
            CanvasEditorMessage::Delete => {
                let (line, col) = self.cursor;
                self.buffer.delete_forward(line, col);
                self.reset_cursor_blink();
                self.cache.clear();
                Task::none()
            }
            CanvasEditorMessage::Enter => {
                let (line, col) = self.cursor;
                self.buffer.insert_newline(line, col);
                self.cursor = (line + 1, 0);
                self.reset_cursor_blink();
                self.cache.clear();
                self.scroll_to_cursor()
            }
            CanvasEditorMessage::ArrowKey(direction, shift_pressed) => {
                if *shift_pressed {
                    // Start selection if not already started
                    if self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor);
                    }
                    self.move_cursor(*direction);
                    self.selection_end = Some(self.cursor);
                } else {
                    // Clear selection and move cursor
                    self.clear_selection();
                    self.move_cursor(*direction);
                }
                self.reset_cursor_blink();
                self.cache.clear();
                self.scroll_to_cursor()
            }
            CanvasEditorMessage::MouseClick(point) => {
                self.handle_mouse_click(*point);
                self.reset_cursor_blink();
                // Clear selection on click
                self.clear_selection();
                self.is_dragging = true;
                self.selection_start = Some(self.cursor);
                Task::none()
            }
            CanvasEditorMessage::MouseDrag(point) => {
                if self.is_dragging {
                    self.handle_mouse_drag(*point);
                    self.cache.clear();
                }
                Task::none()
            }
            CanvasEditorMessage::MouseRelease => {
                self.is_dragging = false;
                Task::none()
            }
            CanvasEditorMessage::Copy => self.copy_selection(),
            CanvasEditorMessage::Paste(text) => {
                // If text is empty, we need to read from clipboard
                if text.is_empty() {
                    // Return a task that reads clipboard and chains to paste
                    iced::clipboard::read().and_then(|clipboard_text| {
                        Task::done(CanvasEditorMessage::Paste(clipboard_text))
                    })
                } else {
                    // We have the text, paste it
                    self.paste_text(text);
                    self.cache.clear();
                    self.scroll_to_cursor()
                }
            }
            CanvasEditorMessage::Tick => {
                // Handle cursor blinking
                if self.last_blink.elapsed() >= CURSOR_BLINK_INTERVAL {
                    self.cursor_visible = !self.cursor_visible;
                    self.last_blink = Instant::now();
                    self.cache.clear();
                }
                Task::none()
            }
            CanvasEditorMessage::PageUp => {
                self.page_up();
                self.reset_cursor_blink();
                self.scroll_to_cursor()
            }
            CanvasEditorMessage::PageDown => {
                self.page_down();
                self.reset_cursor_blink();
                self.scroll_to_cursor()
            }
            CanvasEditorMessage::Home(shift_pressed) => {
                if *shift_pressed {
                    // Start selection if not already started
                    if self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor);
                    }
                    self.cursor.1 = 0; // Move to start of line
                    self.selection_end = Some(self.cursor);
                } else {
                    // Clear selection and move cursor
                    self.clear_selection();
                    self.cursor.1 = 0;
                }
                self.reset_cursor_blink();
                self.cache.clear();
                Task::none()
            }
            CanvasEditorMessage::End(shift_pressed) => {
                let line = self.cursor.0;
                let line_len = self.buffer.line_len(line);

                if *shift_pressed {
                    // Start selection if not already started
                    if self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor);
                    }
                    self.cursor.1 = line_len; // Move to end of line
                    self.selection_end = Some(self.cursor);
                } else {
                    // Clear selection and move cursor
                    self.clear_selection();
                    self.cursor.1 = line_len;
                }
                self.reset_cursor_blink();
                self.cache.clear();
                Task::none()
            }
            CanvasEditorMessage::Scrolled(viewport) => {
                // Track viewport scroll position and height
                self.viewport_scroll = viewport.absolute_offset().y;
                self.viewport_height = viewport.bounds().height;
                Task::none()
            }
        }
    }

    /// Moves the cursor based on arrow key direction.
    fn move_cursor(&mut self, direction: ArrowDirection) {
        let (line, col) = self.cursor;

        match direction {
            ArrowDirection::Up => {
                if line > 0 {
                    let new_line = line - 1;
                    let line_len = self.buffer.line_len(new_line);
                    self.cursor = (new_line, col.min(line_len));
                }
            }
            ArrowDirection::Down => {
                if line + 1 < self.buffer.line_count() {
                    let new_line = line + 1;
                    let line_len = self.buffer.line_len(new_line);
                    self.cursor = (new_line, col.min(line_len));
                }
            }
            ArrowDirection::Left => {
                if col > 0 {
                    self.cursor.1 -= 1;
                } else if line > 0 {
                    // Move to end of previous line
                    let prev_line_len = self.buffer.line_len(line - 1);
                    self.cursor = (line - 1, prev_line_len);
                }
            }
            ArrowDirection::Right => {
                let line_len = self.buffer.line_len(line);
                if col < line_len {
                    self.cursor.1 += 1;
                } else if line + 1 < self.buffer.line_count() {
                    // Move to start of next line
                    self.cursor = (line + 1, 0);
                }
            }
        }
        self.cache.clear();
    }

    /// Handles mouse click to position cursor.
    fn handle_mouse_click(&mut self, point: Point) {
        // Account for gutter width
        if point.x < GUTTER_WIDTH {
            return; // Clicked in gutter
        }

        // Calculate line number
        let line = ((point.y + self.scroll_offset) / LINE_HEIGHT) as usize;
        let line = line.min(self.buffer.line_count().saturating_sub(1));

        // Calculate column
        let x_in_text = point.x - GUTTER_WIDTH;
        let col = (x_in_text / CHAR_WIDTH) as usize;
        let line_len = self.buffer.line_len(line);
        let col = col.min(line_len);

        self.cursor = (line, col);
        self.cache.clear();
    }

    /// Resets cursor blink timer and makes cursor visible.
    fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.last_blink = Instant::now();
    }

    /// Returns a scroll command to make the cursor visible.
    fn scroll_to_cursor(&self) -> Task<CanvasEditorMessage> {
        let cursor_y = self.cursor.0 as f32 * LINE_HEIGHT;
        let viewport_top = self.viewport_scroll;
        let viewport_bottom = self.viewport_scroll + self.viewport_height;

        // Add margins to avoid cursor being exactly at edge
        let top_margin = LINE_HEIGHT * 2.0;
        let bottom_margin = LINE_HEIGHT * 2.0;

        // Calculate new scroll position if cursor is outside visible area
        let new_scroll = if cursor_y < viewport_top + top_margin {
            // Cursor is above viewport - scroll up
            (cursor_y - top_margin).max(0.0)
        } else if cursor_y + LINE_HEIGHT > viewport_bottom - bottom_margin {
            // Cursor is below viewport - scroll down
            cursor_y + LINE_HEIGHT + bottom_margin - self.viewport_height
        } else {
            // Cursor is visible - no scroll needed
            return Task::none();
        };

        scroll_to(
            self.scrollable_id.clone(),
            scrollable::AbsoluteOffset {
                x: 0.0,
                y: new_scroll,
            },
        )
    }

    /// Moves cursor up by one page (approximately viewport height).
    fn page_up(&mut self) {
        let lines_per_page = (self.viewport_height / LINE_HEIGHT) as usize;

        let current_line = self.cursor.0;
        let new_line = current_line.saturating_sub(lines_per_page);
        let line_len = self.buffer.line_len(new_line);

        self.cursor = (new_line, self.cursor.1.min(line_len));
        self.cache.clear();
    }

    /// Moves cursor down by one page (approximately viewport height).
    fn page_down(&mut self) {
        let lines_per_page = (self.viewport_height / LINE_HEIGHT) as usize;

        let current_line = self.cursor.0;
        let max_line = self.buffer.line_count().saturating_sub(1);
        let new_line = (current_line + lines_per_page).min(max_line);
        let line_len = self.buffer.line_len(new_line);

        self.cursor = (new_line, self.cursor.1.min(line_len));
        self.cache.clear();
    }

    /// Handles mouse drag for text selection.
    fn handle_mouse_drag(&mut self, point: Point) {
        // Account for gutter width
        if point.x < GUTTER_WIDTH {
            return;
        }

        // Calculate line and column (same as mouse click)
        let line = ((point.y + self.scroll_offset) / LINE_HEIGHT) as usize;
        let line = line.min(self.buffer.line_count().saturating_sub(1));

        let x_in_text = point.x - GUTTER_WIDTH;
        let col = (x_in_text / CHAR_WIDTH) as usize;
        let line_len = self.buffer.line_len(line);
        let col = col.min(line_len);

        // Update cursor and selection end
        self.cursor = (line, col);
        self.selection_end = Some(self.cursor);
    }

    /// Clears the current selection.
    fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
        self.cache.clear();
    }

    /// Returns the selected text range in normalized order (start before end).
    fn get_selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            // Normalize: ensure start comes before end
            if start.0 < end.0 || (start.0 == end.0 && start.1 < end.1) {
                Some((start, end))
            } else {
                Some((end, start))
            }
        } else {
            None
        }
    }

    /// Returns the selected text as a string.
    fn get_selected_text(&self) -> Option<String> {
        let (start, end) = self.get_selection_range()?;

        if start == end {
            return None; // No selection
        }

        let mut result = String::new();

        if start.0 == end.0 {
            // Single line selection
            let line = self.buffer.line(start.0);
            result.push_str(&line[start.1..end.1]);
        } else {
            // Multi-line selection
            // First line
            let first_line = self.buffer.line(start.0);
            result.push_str(&first_line[start.1..]);
            result.push('\n');

            // Middle lines
            for line_idx in (start.0 + 1)..end.0 {
                result.push_str(self.buffer.line(line_idx));
                result.push('\n');
            }

            // Last line
            let last_line = self.buffer.line(end.0);
            result.push_str(&last_line[..end.1]);
        }

        Some(result)
    }

    /// Copies selected text to clipboard.
    fn copy_selection(&self) -> Task<CanvasEditorMessage> {
        if let Some(text) = self.get_selected_text() {
            iced::clipboard::write(text)
        } else {
            Task::none()
        }
    }

    /// Deletes the selected text.
    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.get_selection_range() {
            if start == end {
                return; // No selection
            }

            // Delete character by character from end to start
            // This is simpler than implementing range deletion in TextBuffer
            for line_idx in (start.0..=end.0).rev() {
                if line_idx == start.0 && line_idx == end.0 {
                    // Single line selection
                    for _ in start.1..end.1 {
                        self.buffer.delete_forward(start.0, start.1);
                    }
                } else if line_idx == start.0 {
                    // First line - delete from start to end of line
                    let line_len = self.buffer.line_len(line_idx);
                    for _ in start.1..line_len {
                        self.buffer.delete_forward(line_idx, start.1);
                    }
                    // Delete newline
                    if line_idx < self.buffer.line_count() {
                        self.buffer.delete_forward(line_idx, start.1);
                    }
                } else if line_idx == end.0 {
                    // Last line - delete from start to end position
                    for _ in 0..end.1 {
                        self.buffer.delete_forward(start.0, start.1);
                    }
                } else {
                    // Middle line - delete entire line
                    let line_len = self.buffer.line_len(start.0);
                    for _ in 0..line_len {
                        self.buffer.delete_forward(start.0, start.1);
                    }
                    // Delete newline
                    if start.0 < self.buffer.line_count() {
                        self.buffer.delete_forward(start.0, start.1);
                    }
                }
            }

            // Move cursor to selection start
            self.cursor = start;
            self.clear_selection();
        }
    }

    /// Pastes text from clipboard at cursor position.
    fn paste_text(&mut self, text: &str) {
        // If there's a selection, delete it first
        if self.selection_start.is_some() && self.selection_end.is_some() {
            self.delete_selection();
        }

        // Insert text character by character
        for ch in text.chars() {
            if ch == '\n' {
                let (line, col) = self.cursor;
                self.buffer.insert_newline(line, col);
                self.cursor = (line + 1, 0);
            } else {
                let (line, col) = self.cursor;
                self.buffer.insert_char(line, col, ch);
                self.cursor.1 += 1;
            }
        }
    }

    /// Returns the editor content as a string.
    #[must_use]
    pub fn content(&self) -> String {
        self.buffer.to_string()
    }

    /// Creates the view element with scrollable wrapper.
    pub fn view(&self) -> Element<'_, CanvasEditorMessage> {
        // Calculate total content height
        let total_lines = self.buffer.line_count();
        let content_height = total_lines as f32 * LINE_HEIGHT;

        // Create canvas with fixed height based on content
        let canvas = Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fixed(content_height));

        // Capture theme colors for the scrollbar style closure
        let scrollbar_bg = self.theme.scrollbar_background;
        let scroller_color = self.theme.scroller_color;

        // Wrap in scrollable for automatic scrollbar display with custom style
        Scrollable::new(canvas)
            .id(self.scrollable_id.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .on_scroll(CanvasEditorMessage::Scrolled)
            .style(move |_theme, _status| scrollable::Style {
                container: container::Style::default(),
                vertical_rail: scrollable::Rail {
                    background: Some(scrollbar_bg.into()),
                    border: Border {
                        radius: 4.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    scroller: scrollable::Scroller {
                        background: scroller_color.into(),
                        border: Border {
                            radius: 4.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                    },
                },
                horizontal_rail: scrollable::Rail {
                    background: Some(scrollbar_bg.into()),
                    border: Border {
                        radius: 4.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    scroller: scrollable::Scroller {
                        background: scroller_color.into(),
                        border: Border {
                            radius: 4.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                    },
                },
                gap: None,
                auto_scroll: scrollable::AutoScroll {
                    background: Color::TRANSPARENT.into(),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    icon: Color::TRANSPARENT,
                },
            })
            .into()
    }
}

impl canvas::Program<CanvasEditorMessage> for CanvasEditor {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Background
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), self.theme.background);

            // Calculate visible lines (virtual scrolling - Scrollable handles the offset)
            // Since the canvas has full height, we need to draw all lines
            let total_lines = self.buffer.line_count();

            // Draw gutter background (full height)
            frame.fill_rectangle(
                Point::ORIGIN,
                Size::new(GUTTER_WIDTH, bounds.height),
                self.theme.gutter_background,
            );

            // Load syntax highlighting
            let syntax_set = SyntaxSet::load_defaults_newlines();
            let theme_set = ThemeSet::load_defaults();
            let syntax_theme = &theme_set.themes["base16-ocean.dark"];

            let syntax_ref = match self.syntax.as_str() {
                "py" | "python" => syntax_set.find_syntax_by_extension("py"),
                "lua" => syntax_set.find_syntax_by_extension("lua"),
                "rs" | "rust" => syntax_set.find_syntax_by_extension("rs"),
                "js" | "javascript" => syntax_set.find_syntax_by_extension("js"),
                _ => Some(syntax_set.find_syntax_plain_text()),
            };

            // Draw line numbers and text for all lines (Scrollable clips viewport)
            for line_idx in 0..total_lines {
                let y = line_idx as f32 * LINE_HEIGHT;

                // Draw line number
                let line_num_text = format!("{:>4}", line_idx + 1);
                frame.fill_text(canvas::Text {
                    content: line_num_text,
                    position: Point::new(5.0, y + 2.0),
                    color: self.theme.line_number_color,
                    size: FONT_SIZE.into(),
                    font: iced::Font::MONOSPACE,
                    ..canvas::Text::default()
                });

                // Highlight current line
                if line_idx == self.cursor.0 {
                    frame.fill_rectangle(
                        Point::new(GUTTER_WIDTH, y),
                        Size::new(bounds.width - GUTTER_WIDTH, LINE_HEIGHT),
                        Color::from_rgb(0.15, 0.15, 0.2),
                    );
                }

                // Draw text content with syntax highlighting
                let line_content = self.buffer.line(line_idx);

                if let Some(syntax) = syntax_ref {
                    let mut highlighter = HighlightLines::new(syntax, syntax_theme);
                    let ranges = highlighter
                        .highlight_line(line_content, &syntax_set)
                        .unwrap_or_else(|_| vec![(Style::default(), line_content)]);

                    let mut x_offset = GUTTER_WIDTH + 5.0;
                    for (style, text) in ranges {
                        let color = Color::from_rgb(
                            f32::from(style.foreground.r) / 255.0,
                            f32::from(style.foreground.g) / 255.0,
                            f32::from(style.foreground.b) / 255.0,
                        );

                        frame.fill_text(canvas::Text {
                            content: text.to_string(),
                            position: Point::new(x_offset, y + 2.0),
                            color,
                            size: FONT_SIZE.into(),
                            font: iced::Font::MONOSPACE,
                            ..canvas::Text::default()
                        });

                        x_offset += text.len() as f32 * CHAR_WIDTH;
                    }
                } else {
                    // Fallback to plain text
                    frame.fill_text(canvas::Text {
                        content: line_content.to_string(),
                        position: Point::new(GUTTER_WIDTH + 5.0, y + 2.0),
                        color: self.theme.text_color,
                        size: FONT_SIZE.into(),
                        font: iced::Font::MONOSPACE,
                        ..canvas::Text::default()
                    });
                }
            }

            // Draw selection highlight
            if let Some((start, end)) = self.get_selection_range() {
                if start == end {
                    // No selection, do nothing
                } else {
                    let selection_color = Color {
                        r: 0.3,
                        g: 0.5,
                        b: 0.8,
                        a: 0.3,
                    };

                    if start.0 == end.0 {
                        // Single line selection
                        let y = start.0 as f32 * LINE_HEIGHT;
                        let x_start = GUTTER_WIDTH + 5.0 + start.1 as f32 * CHAR_WIDTH;
                        let x_end = GUTTER_WIDTH + 5.0 + end.1 as f32 * CHAR_WIDTH;

                        frame.fill_rectangle(
                            Point::new(x_start, y + 2.0),
                            Size::new(x_end - x_start, LINE_HEIGHT - 4.0),
                            selection_color,
                        );
                    } else {
                        // Multi-line selection
                        // First line - from start column to end of line
                        let y_start = start.0 as f32 * LINE_HEIGHT;
                        let x_start = GUTTER_WIDTH + 5.0 + start.1 as f32 * CHAR_WIDTH;
                        let first_line_len = self.buffer.line_len(start.0);
                        let x_end_first = GUTTER_WIDTH + 5.0 + first_line_len as f32 * CHAR_WIDTH;

                        frame.fill_rectangle(
                            Point::new(x_start, y_start + 2.0),
                            Size::new(x_end_first - x_start, LINE_HEIGHT - 4.0),
                            selection_color,
                        );

                        // Middle lines - full width
                        for line_idx in (start.0 + 1)..end.0 {
                            let y = line_idx as f32 * LINE_HEIGHT;
                            let line_len = self.buffer.line_len(line_idx);
                            let width = line_len as f32 * CHAR_WIDTH;

                            frame.fill_rectangle(
                                Point::new(GUTTER_WIDTH + 5.0, y + 2.0),
                                Size::new(width, LINE_HEIGHT - 4.0),
                                selection_color,
                            );
                        }

                        // Last line - from start of line to end column
                        let y_end = end.0 as f32 * LINE_HEIGHT;
                        let x_end = GUTTER_WIDTH + 5.0 + end.1 as f32 * CHAR_WIDTH;

                        frame.fill_rectangle(
                            Point::new(GUTTER_WIDTH + 5.0, y_end + 2.0),
                            Size::new(x_end - (GUTTER_WIDTH + 5.0), LINE_HEIGHT - 4.0),
                            selection_color,
                        );
                    }
                }
            }

            // Draw cursor
            if self.cursor_visible {
                let cursor_x = GUTTER_WIDTH + 5.0 + self.cursor.1 as f32 * CHAR_WIDTH;
                let cursor_y = self.cursor.0 as f32 * LINE_HEIGHT;

                frame.fill_rectangle(
                    Point::new(cursor_x, cursor_y + 2.0),
                    Size::new(2.0, LINE_HEIGHT - 4.0),
                    self.theme.text_color,
                );
            }
        });

        vec![geometry]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<CanvasEditorMessage>> {
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                // Handle Ctrl+C (copy)
                if modifiers.control()
                    && matches!(key, keyboard::Key::Character(c) if c.as_str() == "c")
                {
                    return Some(Action::publish(CanvasEditorMessage::Copy).and_capture());
                }

                // Handle Ctrl+V (paste) - read clipboard and send paste message
                if modifiers.control()
                    && matches!(key, keyboard::Key::Character(v) if v.as_str() == "v")
                {
                    // Return an action that requests clipboard read
                    return Some(Action::publish(CanvasEditorMessage::Paste(String::new())));
                }

                let message = match key {
                    keyboard::Key::Character(c) if !modifiers.control() => {
                        c.chars().next().map(CanvasEditorMessage::CharacterInput)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                        Some(CanvasEditorMessage::Backspace)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Delete) => {
                        Some(CanvasEditorMessage::Delete)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        Some(CanvasEditorMessage::Enter)
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(
                        CanvasEditorMessage::ArrowKey(ArrowDirection::Up, modifiers.shift()),
                    ),
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(
                        CanvasEditorMessage::ArrowKey(ArrowDirection::Down, modifiers.shift()),
                    ),
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => Some(
                        CanvasEditorMessage::ArrowKey(ArrowDirection::Left, modifiers.shift()),
                    ),
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => Some(
                        CanvasEditorMessage::ArrowKey(ArrowDirection::Right, modifiers.shift()),
                    ),
                    keyboard::Key::Named(keyboard::key::Named::PageUp) => {
                        Some(CanvasEditorMessage::PageUp)
                    }
                    keyboard::Key::Named(keyboard::key::Named::PageDown) => {
                        Some(CanvasEditorMessage::PageDown)
                    }
                    keyboard::Key::Named(keyboard::key::Named::Home) => {
                        Some(CanvasEditorMessage::Home(modifiers.shift()))
                    }
                    keyboard::Key::Named(keyboard::key::Named::End) => {
                        Some(CanvasEditorMessage::End(modifiers.shift()))
                    }
                    _ => None,
                };

                message.map(|msg| Action::publish(msg).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                cursor.position_in(bounds).map(|position| {
                    // Don't capture the event so it can bubble up for focus management
                    Action::publish(CanvasEditorMessage::MouseClick(position))
                })
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Handle mouse drag for selection
                cursor.position_in(bounds).map(|position| {
                    Action::publish(CanvasEditorMessage::MouseDrag(position)).and_capture()
                })
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(Action::publish(CanvasEditorMessage::MouseRelease).and_capture())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_canvas_editor() {
        let editor = CanvasEditor::new("line1\nline2", "py");
        assert_eq!(editor.cursor, (0, 0));
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = CanvasEditor::new("line1\nline2", "py");
        editor.move_cursor(ArrowDirection::Down);
        assert_eq!(editor.cursor.0, 1);
        editor.move_cursor(ArrowDirection::Right);
        assert_eq!(editor.cursor.1, 1);
    }

    #[test]
    fn test_home_key() {
        let mut editor = CanvasEditor::new("hello world", "py");
        editor.cursor = (0, 5); // Move to middle of line
        let _ = editor.update(&CanvasEditorMessage::Home(false));
        assert_eq!(editor.cursor, (0, 0));
    }

    #[test]
    fn test_end_key() {
        let mut editor = CanvasEditor::new("hello world", "py");
        editor.cursor = (0, 0);
        let _ = editor.update(&CanvasEditorMessage::End(false));
        assert_eq!(editor.cursor, (0, 11)); // Length of "hello world"
    }

    #[test]
    fn test_page_down() {
        // Create editor with many lines
        let content = (0..100)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut editor = CanvasEditor::new(&content, "py");

        let _ = editor.update(&CanvasEditorMessage::PageDown);
        // Should move approximately 30 lines (600px / 20px per line)
        assert!(editor.cursor.0 >= 25);
        assert!(editor.cursor.0 <= 35);
    }

    #[test]
    fn test_page_up() {
        // Create editor with many lines
        let content = (0..100)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut editor = CanvasEditor::new(&content, "py");

        // Move to line 50
        editor.cursor = (50, 0);
        let _ = editor.update(&CanvasEditorMessage::PageUp);

        // Should move approximately 30 lines up
        assert!(editor.cursor.0 >= 15);
        assert!(editor.cursor.0 <= 25);
    }

    #[test]
    fn test_page_down_at_end() {
        let content = (0..10)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut editor = CanvasEditor::new(&content, "py");

        let _ = editor.update(&CanvasEditorMessage::PageDown);
        // Should be at last line (line 9)
        assert_eq!(editor.cursor.0, 9);
    }

    #[test]
    fn test_page_up_at_start() {
        let content = (0..100)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut editor = CanvasEditor::new(&content, "py");

        // Already at start
        editor.cursor = (0, 0);
        let _ = editor.update(&CanvasEditorMessage::PageUp);
        assert_eq!(editor.cursor.0, 0);
    }
}
