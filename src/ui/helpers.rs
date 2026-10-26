use gpui::{div, prelude::*, px, rgb, Context, IntoElement, Render, Window};

pub fn render_field_value_with_cursor(
    text: &str,
    is_active: bool,
    cursor_char_pos: usize,
    placeholder: &str,
    is_password: bool,
    is_dark: bool,
) -> impl IntoElement {
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x18181b) };
    let placeholder_color = if is_dark { rgb(0x71717a) } else { rgb(0xa1a1aa) };
    let cursor_color = if is_dark { rgb(0x3b82f6) } else { rgb(0x2563eb) };

    let display_text = if is_password {
        "•".repeat(text.chars().count())
    } else {
        text.to_string()
    };

    if !is_active {
        if display_text.is_empty() {
            return div()
                .text_xs()
                .text_color(placeholder_color)
                .child(placeholder.to_string());
        } else {
            return div()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_color)
                .child(display_text);
        }
    }

    let char_count = display_text.chars().count();
    let clamped_pos = cursor_char_pos.min(char_count);

    let before: String = display_text.chars().take(clamped_pos).collect();
    let after: String = display_text.chars().skip(clamped_pos).collect();
    let show_placeholder = display_text.is_empty() && !placeholder.is_empty();

    div()
        .flex()
        .items_center()
        .text_xs()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(text_color)
        .child(before)
        .child(
            div()
                .w(px(1.5))
                .h(px(13.0))
                .bg(cursor_color)
                .rounded(px(0.5))
                .mx(px(0.5)),
        )
        .when(show_placeholder, |el| {
            el.child(
                div()
                    .text_color(placeholder_color)
                    .font_weight(gpui::FontWeight::NORMAL)
                    .child(placeholder.to_string()),
            )
        })
        .when(!after.is_empty(), |el| el.child(after))
}

pub fn render_textarea_value_with_cursor(
    text: &str,
    is_active: bool,
    cursor_char_pos: usize,
    placeholder: &str,
    is_dark: bool,
) -> impl IntoElement {
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x18181b) };
    let placeholder_color = if is_dark { rgb(0x71717a) } else { rgb(0xa1a1aa) };
    let cursor_color = if is_dark { rgb(0x3b82f6) } else { rgb(0x2563eb) };

    let char_count = text.chars().count();
    let clamped_pos = cursor_char_pos.min(char_count);

    if !is_active {
        if text.trim().is_empty() {
            return div()
                .text_xs()
                .text_color(placeholder_color)
                .child(placeholder.to_string());
        } else {
            return div()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_color)
                .child(text.to_string());
        }
    }

    if text.is_empty() {
        return div()
            .text_xs()
            .child(
                div()
                    .text_color(cursor_color)
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("|"),
            );
    }

    let before: String = text.chars().take(clamped_pos).collect();
    let after: String = text.chars().skip(clamped_pos).collect();
    let mut combined = before;
    combined.push('|');
    combined.push_str(&after);

    div()
        .text_xs()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(text_color)
        .child(combined)
}

pub struct FileTooltip {
    pub text: String,
    pub is_dark: bool,
}

impl Render for FileTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .mt(px(14.0))
            .px_2p5()
            .py_1p5()
            .max_w(px(500.0))
            .rounded(px(5.0))
            .bg(if self.is_dark { rgb(0x222227) } else { rgb(0xffffff) })
            .border_1()
            .border_color(if self.is_dark { rgb(0x3f3f46) } else { rgb(0xd4d4d8) })
            .shadow_lg()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(if self.is_dark { rgb(0xf4f4f5) } else { rgb(0x18181b) })
            .whitespace_normal()
            .child(self.text.clone())
    }
}

