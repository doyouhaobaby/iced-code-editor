use iced::Color;

/// Theme configuration for the code editor.
///
/// Defines colors for various editor components including background,
/// text, line numbers, gutter, and scrollbar.
#[derive(Debug, Clone)]
pub struct EditorTheme {
    /// Main editor background color
    pub background: Color,
    /// Text content color
    pub text_color: Color,
    /// Line numbers gutter background color
    pub gutter_background: Color,
    /// Border color for the gutter
    pub gutter_border: Color,
    /// Color for line numbers text
    pub line_number_color: Color,
    /// Scrollbar background color
    pub scrollbar_background: Color,
    /// Scrollbar scroller (thumb) color
    pub scroller_color: Color,
}

impl EditorTheme {
    /// Creates a dark theme with VSCode-like colors.
    ///
    /// # Returns
    ///
    /// A dark theme suitable for low-light environments.
    pub fn dark() -> Self {
        Self {
            background: Color::from_rgb(0.05, 0.05, 0.07),
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            gutter_background: Color::from_rgb(0.08, 0.08, 0.10),
            gutter_border: Color::from_rgb(0.15, 0.15, 0.15),
            line_number_color: Color::from_rgb(0.5, 0.5, 0.5),
            scrollbar_background: Color::from_rgb(0.1, 0.1, 0.12),
            scroller_color: Color::from_rgb(0.3, 0.3, 0.35),
        }
    }
}
