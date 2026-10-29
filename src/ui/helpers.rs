use gpui::{div, prelude::*, px, rgb, Context, IntoElement, Render, Window};
use crate::i18n::I18n;
use crate::icons::render_icon;
use crate::model::{format_bytes, FileEntry};

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

#[derive(Clone, Debug)]
pub struct FileTooltipItem {
    pub label: &'static str,
    pub value: String,
    pub is_full_width: bool,
}

#[derive(Clone, Debug)]
pub struct FileTooltipDetails {
    pub name: String,
    pub icon_path: &'static str,
    pub icon_color: u32,
    pub items: Vec<FileTooltipItem>,
}

impl FileTooltipDetails {
    pub fn from_entry(entry: &FileEntry, t: &I18n, icon_color: u32) -> Self {
        let (icon_path, _) = crate::icons::get_file_icon(&entry.name, entry.is_dir, entry.is_symlink);
        let mut items = Vec::new();

        // 1. Type
        let type_str = if entry.is_symlink {
            t.symlink_type_label().to_string()
        } else if entry.is_dir {
            t.folder_type_label().to_string()
        } else {
            let ext = entry.name.rfind('.').map(|p| &entry.name[p + 1..]).unwrap_or("");
            if ext.is_empty() {
                t.file_type_label().to_string()
            } else {
                format!("{} ({})", t.file_type_label(), ext.to_uppercase())
            }
        };
        items.push(FileTooltipItem {
            label: t.tooltip_type(),
            value: type_str,
            is_full_width: false,
        });

        // 2. Size
        if entry.is_dir {
            if let Some(calc) = entry.calculated_dir_size {
                items.push(FileTooltipItem {
                    label: t.col_size(),
                    value: format!("{} ({} B)", format_bytes(calc), calc),
                    is_full_width: false,
                });
            }
        } else {
            items.push(FileTooltipItem {
                label: t.col_size(),
                value: format!("{} ({} B)", format_bytes(entry.size), entry.size),
                is_full_width: false,
            });
        }

        // 3. Permissions
        let perm_str = if entry.octal_mode > 0 {
            format!("{} ({:04o})", entry.permissions_str, entry.octal_mode)
        } else {
            entry.permissions_str.clone()
        };
        items.push(FileTooltipItem {
            label: t.col_permissions(),
            value: perm_str,
            is_full_width: false,
        });

        // 4. Owner
        if !entry.owner.is_empty() {
            items.push(FileTooltipItem {
                label: t.tooltip_owner(),
                value: entry.owner.clone(),
                is_full_width: false,
            });
        }

        // 5. Created
        if let Some(ref cr) = entry.created {
            if !cr.is_empty() && cr != "-" {
                items.push(FileTooltipItem {
                    label: t.tooltip_created(),
                    value: cr.clone(),
                    is_full_width: false,
                });
            }
        }

        // 6. Modified
        if !entry.modified.is_empty() && entry.modified != "-" {
            items.push(FileTooltipItem {
                label: t.col_modified(),
                value: entry.modified.clone(),
                is_full_width: false,
            });
        }

        // 7. Path
        if !entry.path.is_empty() {
            items.push(FileTooltipItem {
                label: t.tooltip_path(),
                value: entry.path.clone(),
                is_full_width: true,
            });
        }

        FileTooltipDetails {
            name: entry.name.clone(),
            icon_path,
            icon_color,
            items,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileTooltip {
    pub text: String,
    pub details: Option<FileTooltipDetails>,
    pub is_dark: bool,
}

impl FileTooltip {
    pub fn simple(text: String, is_dark: bool) -> Self {
        Self {
            text,
            details: None,
            is_dark,
        }
    }

    pub fn from_entry(entry: &FileEntry, t: &I18n, is_dark: bool, icon_color: u32) -> Self {
        Self {
            text: entry.name.clone(),
            details: Some(FileTooltipDetails::from_entry(entry, t, icon_color)),
            is_dark,
        }
    }
}

impl Render for FileTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = self.is_dark;
        let bg = if is_dark { rgb(0x1e1e24) } else { rgb(0xffffff) };
        let border = if is_dark { rgb(0x3f3f46) } else { rgb(0xd4d4d8) };
        let text_main = if is_dark { rgb(0xf4f4f5) } else { rgb(0x18181b) };
        let text_muted = if is_dark { rgb(0xa1a1aa) } else { rgb(0x71717a) };

        if let Some(ref details) = self.details {
            div()
                .mt(px(26.0))
                .ml(px(8.0))
                .px_3()
                .py_2()
                .w(px(380.0))
                .rounded(px(6.0))
                .bg(bg)
                .border_1()
                .border_color(border)
                .shadow_lg()
                .flex()
                .flex_col()
                .gap_1p5()
                // Header: Icon + Name (wraps if too long, never overflows)
                .child(
                    div()
                        .w_full()
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(
                            div()
                                .mt(px(1.0))
                                .flex_shrink_0()
                                .child(render_icon(details.icon_path, 15.0, details.icon_color)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_xs()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(text_main)
                                .whitespace_normal()
                                .child(details.name.clone()),
                        ),
                )
                // Divider
                .child(div().h(px(1.0)).bg(if is_dark { rgb(0x2e2e34) } else { rgb(0xe4e4e7) }))
                // Key-value pairs
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(details.items.iter().map(|item| {
                            if item.is_full_width {
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .text_xs()
                                    .child(
                                        div()
                                            .text_color(text_muted)
                                            .child(format!("{}:", item.label)),
                                    )
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(text_main)
                                            .whitespace_normal()
                                            .child(item.value.clone()),
                                    )
                            } else {
                                div()
                                    .flex()
                                    .items_start()
                                    .justify_between()
                                    .gap_2()
                                    .text_xs()
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .text_color(text_muted)
                                            .child(format!("{}:", item.label)),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .text_right()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(text_main)
                                            .whitespace_normal()
                                            .child(item.value.clone()),
                                    )
                            }
                        })),
                )
        } else {
            div()
                .mt(px(26.0))
                .ml(px(8.0))
                .px_2p5()
                .py_1p5()
                .w(px(320.0))
                .rounded(px(5.0))
                .bg(if self.is_dark { rgb(0x222227) } else { rgb(0xffffff) })
                .border_1()
                .border_color(border)
                .shadow_lg()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_main)
                .whitespace_normal()
                .child(self.text.clone())
        }
    }
}

