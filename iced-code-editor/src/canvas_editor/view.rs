//! Iced UI view and rendering logic.

use iced::advanced::input_method;
use iced::advanced::widget::{Widget, tree};
use iced::widget::canvas::Canvas;
use iced::widget::{Row, Scrollable, Space, container, scrollable};
use iced::{
    Background, Border, Color, Element, Event, Length, Rectangle, Shadow,
};
use iced::{Size, Vector, mouse, window};
use iced::advanced::{Renderer, Shell};

use super::search_dialog;
use super::wrapping::WrappingCalculator;
use super::{CodeEditor, GUTTER_WIDTH, LINE_HEIGHT, Message};

#[derive(Debug, Clone)]
struct ImeRequester {
    // -------------------------------------------------------------------------
    // 输入法请求器状态字段说明
    // -------------------------------------------------------------------------
    
    // 是否启用输入法
    // 逻辑：只有当编辑器同时拥有 Iced 焦点（is_focused）和内部画布焦点（has_canvas_focus）时，
    // 该值才为 true。这对应了 `shell.request_input_method` 的 Enabled/Disabled 状态。
    enabled: bool,

    // 输入法光标矩形（Caret Rectangle）
    // 作用：告诉操作系统当前光标在屏幕上的确切位置（x, y, w, h）。
    // 系统利用此信息将输入法候选窗口（Candidate Window）定位在光标旁边，
    // 避免候选窗口遮挡光标或出现在屏幕角落（即 "Over-the-spot" 风格）。
    cursor: Rectangle,

    // 当前预编辑内容
    // 作用：将当前的预编辑文本（如 "nihao"）回传给 Shell，
    // 虽然通常是 Shell 发给 View，但这里作为状态保持，确保请求一致性。
    preedit: Option<input_method::Preedit<String>>,
}

impl ImeRequester {
    fn new(
        enabled: bool,
        cursor: Rectangle,
        preedit: Option<input_method::Preedit<String>>,
    ) -> Self {
        Self { enabled, cursor, preedit }
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for ImeRequester
where
    iced::Renderer: Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        _tree: &mut tree::Tree,
        _renderer: &iced::Renderer,
        _limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        iced::advanced::layout::Node::new(Size::new(0.0, 0.0))
    }

    fn draw(
        &self,
        _tree: &tree::Tree,
        _renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &iced::advanced::renderer::Style,
        _layout: iced::advanced::layout::Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn update(
        &mut self,
        _tree: &mut tree::Tree,
        event: &Event,
        _layout: iced::advanced::layout::Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        // IME 请求核心逻辑
        // ---------------------------------------------------------------------
        // 为什么在 `RedrawRequested` 事件中请求？
        // 1. Iced 的输入法协议要求每一帧（或状态变化时）显式声明输入法状态。
        // 2. `RedrawRequested` 是渲染周期的开始，此时请求能确保系统获得最新的光标位置，
        //    从而让候选窗口紧跟光标移动（尤其是在打字或滚动时）。
        //
        // 逻辑分支：
        // - enabled = true: 编辑器激活且聚焦。请求 `InputMethod::Enabled`，
        //   并附带当前的光标矩形（cursor）和预编辑内容（preedit）。
        // - enabled = false: 编辑器失焦。请求 `InputMethod::Disabled`，
        //   告知系统关闭软键盘或重置输入法状态。
        // ---------------------------------------------------------------------
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            if self.enabled {
                let ime = input_method::InputMethod::Enabled {
                    cursor: self.cursor,
                    purpose: input_method::Purpose::Normal,
                    preedit: self
                        .preedit
                        .as_ref()
                        .map(input_method::Preedit::as_ref),
                };
                shell.request_input_method(&ime);
            } else {
                let disabled: input_method::InputMethod<&str> =
                    input_method::InputMethod::Disabled;
                shell.request_input_method(&disabled);
            }
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &tree::Tree,
        _layout: iced::advanced::layout::Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::None
    }

    fn overlay<'a>(
        &'a mut self,
        _tree: &'a mut tree::Tree,
        _layout: iced::advanced::layout::Layout<'a>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<iced::overlay::Element<'a, Message, iced::Theme, iced::Renderer>>
    {
        None
    }
}

impl CodeEditor {
    /// Creates the view element with scrollable wrapper.
    ///
    /// The backgrounds (editor and gutter) are handled by container styles
    /// to ensure proper clipping when the pane is resized.
    pub fn view(&self) -> Element<'_, Message> {
        // Calculate total content height based on actual lines
        // When wrapping is enabled, use visual line count
        let wrapping_calc =
            WrappingCalculator::new(self.wrap_enabled, self.wrap_column);

        // Use viewport width for calculating visual lines
        let visual_lines = wrapping_calc.calculate_visual_lines(
            &self.buffer,
            self.viewport_width,
            self.gutter_width(),
        );

        let total_visual_lines = visual_lines.len();
        let content_height = total_visual_lines as f32 * LINE_HEIGHT;

        // Use max of content height and viewport height to ensure the canvas
        // always covers the visible area (prevents visual artifacts when
        // content is shorter than viewport after reset/file change)
        let canvas_height = content_height.max(self.viewport_height);

        // Create canvas with height that covers at least the viewport
        let canvas = Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fixed(canvas_height));

        // Capture style colors for closures
        let scrollbar_bg = self.style.scrollbar_background;
        let scroller_color = self.style.scroller_color;
        let background_color = self.style.background;
        let gutter_background = self.style.gutter_background;

        // Wrap in scrollable for automatic scrollbar display with custom style
        // Use Length::Fill to respect parent container constraints and enable proper clipping
        // Background is TRANSPARENT here because it's handled by the Stack layer below
        let scrollable = Scrollable::new(canvas)
            .id(self.scrollable_id.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .on_scroll(Message::Scrolled)
            .style(move |_theme, _status| scrollable::Style {
                container: container::Style {
                    background: Some(Background::Color(Color::TRANSPARENT)),
                    ..container::Style::default()
                },
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
            });

        // Gutter background container (fixed width, clipped by parent)
        // Only create if line numbers are enabled
        let gutter_container = if self.line_numbers_enabled {
            Some(
                container(
                    Space::new().width(Length::Fill).height(Length::Fill),
                )
                .width(Length::Fixed(GUTTER_WIDTH))
                .height(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(Background::Color(gutter_background)),
                    ..container::Style::default()
                }),
            )
        } else {
            None
        };

        // Code background container (fills remaining width)
        let code_background_container =
            container(Space::new().width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(Background::Color(background_color)),
                    ..container::Style::default()
                });

        // Main layout: use a Stack to layer the backgrounds behind the scrollable
        // The scrollable has a transparent background so the colors show through
        let background_row = if let Some(gutter) = gutter_container {
            Row::new().push(gutter).push(code_background_container)
        } else {
            Row::new().push(code_background_container)
        };

        let mut editor_stack = iced::widget::Stack::new()
            .push(
                // Background layer (bottom): gutter + code backgrounds
                background_row,
            )
            .push(
                // Scrollable layer (top) - transparent, overlays the backgrounds
                scrollable,
            );

        let ime_enabled = self.is_focused() && self.has_canvas_focus;
        let cursor_rect = if ime_enabled {
            if let Some(cursor_visual) = WrappingCalculator::logical_to_visual(
                &visual_lines,
                self.cursor.0,
                self.cursor.1,
            ) {
                let vl = &visual_lines[cursor_visual];
                let line_content = self.buffer.line(vl.logical_line);
                let prefix_len = self.cursor.1.saturating_sub(vl.start_col);
                let prefix_text: String = line_content
                    .chars()
                    .skip(vl.start_col)
                    .take(prefix_len)
                    .collect();
                let cursor_x = self.gutter_width()
                    + 5.0
                    + super::measure_text_width(&prefix_text);
                let cursor_y = cursor_visual as f32 * LINE_HEIGHT;
                Rectangle::new(
                    iced::Point::new(cursor_x, cursor_y + 2.0),
                    Size::new(2.0, LINE_HEIGHT - 4.0),
                )
            } else {
                Rectangle::new(iced::Point::new(0.0, 0.0), Size::new(0.0, 0.0))
            }
        } else {
            Rectangle::new(iced::Point::new(0.0, 0.0), Size::new(0.0, 0.0))
        };

        let preedit =
            self.ime_preedit.as_ref().map(|p| input_method::Preedit {
                content: p.content.clone(),
                selection: p.selection.clone(),
                text_size: None,
            });

        // 不可见的 IME 请求层：用于在每次重绘时向 Shell 传递输入法状态与插入点
        // 说明：Canvas 的 Program 无法直接访问 Shell，因此通过该 Widget 间接完成输入法激活
        let ime_layer = ImeRequester::new(ime_enabled, cursor_rect, preedit);
        editor_stack = editor_stack.push(iced::Element::new(ime_layer));

        // Add search dialog overlay if open
        if self.search_state.is_open {
            let search_dialog =
                search_dialog::view(&self.search_state, &self.translations);

            // Position the dialog in top-right corner with 20px margin
            // Use a Row with Fill space to push the dialog to the right
            let positioned_dialog = container(
                Row::new()
                    .push(Space::new().width(Length::Fill)) // Push to right
                    .push(search_dialog),
            )
            .padding(20) // 20px margin from edges
            .width(Length::Fill)
            .height(Length::Shrink);

            editor_stack = editor_stack.push(positioned_dialog);
        }

        // Wrap in a container with clip to ensure proper bounds
        container(editor_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .clip(true)
            .into()
    }
}
