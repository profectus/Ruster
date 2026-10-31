use gpui::{div, prelude::*, px, rgb, rgba, Context, IntoElement, MouseButton};
use crate::app::RusterView;
use crate::diff::DiffLineType;
use crate::icons::render_icon;
use crate::model::{format_bytes, ActiveModal};

impl RusterView {
    pub fn render_file_compare_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();

        let Some(comp) = &self.file_compare else {
            return div();
        };

        let is_identical = comp.are_identical;
        let is_binary = comp.is_binary;
        let diff_count = comp.difference_count;

        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(theme.modal_backdrop())
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.stop_propagation()))
            .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
            .child(
                div()
                    .w(px(900.0))
                    .h(px(640.0))
                    .rounded(px(10.0))
                    .bg(theme.modal_bg())
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_xl()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    // 1. Header
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_4()
                            .py_2p5()
                            .rounded_t(px(10.0))
                            .border_b_1()
                            .border_color(theme.border())
                            .flex()
                            .items_center()
                            .justify_between()
                            .bg(theme.modal_header_bg())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(render_icon("icons/file.svg", 16.0, theme.c_icon_active()))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(t.modal_compare_title()),
                                    ),
                            )
                            .child(
                                div()
                                    .id("diff-close-x")
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded(px(4.0))
                                    .hover(|s| s.bg(theme.popover_item_active()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.active_modal = ActiveModal::None;
                                            this.file_compare = None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/close.svg", 14.0, theme.c_icon_secondary())),
                            ),
                    )
                    // 2. Comparison Metadata Banner
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_4()
                            .py_2p5()
                            .border_b_1()
                            .border_color(theme.border())
                            .bg(theme.modal_card_bg())
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(format!("A: {}", comp.file_a_name)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted())
                                            .child(format_bytes(comp.file_a_size)),
                                    ),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(6.0))
                                    .bg(if is_identical {
                                        rgba(0x10b98120)
                                    } else if is_binary {
                                        rgba(0xf59e0b20)
                                    } else {
                                        rgba(0xef444420)
                                    })
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(if is_identical {
                                        rgb(0x10b981)
                                    } else if is_binary {
                                        rgb(0xf59e0b)
                                    } else {
                                        rgb(0xef4444)
                                    })
                                    .child(if is_identical {
                                        t.lbl_identical()
                                    } else if is_binary {
                                        t.lbl_different()
                                    } else {
                                        t.lbl_different()
                                    }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .items_end()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(format!("B: {}", comp.file_b_name)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted())
                                            .child(format_bytes(comp.file_b_size)),
                                    ),
                            ),
                    )
                    // 3. Diff Content View
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .when(is_binary, |d| {
                                d.child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme.text_secondary())
                                                .child(if is_identical {
                                                    "Binární soubory mají identický obsah."
                                                } else {
                                                    t.lbl_binary_notice()
                                                }),
                                        ),
                                )
                            })
                            .when(!is_binary && is_identical, |d| {
                                d.child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .gap_2()
                                        .child(render_icon("icons/check.svg", 32.0, 0x10b981))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(rgb(0x10b981))
                                                .child(t.lbl_identical()),
                                        ),
                                )
                            })
                            .when(!is_binary && !is_identical, |d| {
                                d.child(
                                    div()
                                        .id("diff-lines-container")
                                        .flex_1()
                                        .overflow_y_scroll()
                                        .font_family("JetBrains Mono, Menlo, monospace")
                                        .text_xs()
                                        .children(
                                            comp.diff_lines.iter().enumerate().map(|(idx, line)| {
                                                let (bg_color, indicator, text_color) = match line.line_type {
                                                    DiffLineType::Added => (
                                                        rgba(0x10b98115),
                                                        "+",
                                                        rgb(0x10b981),
                                                    ),
                                                    DiffLineType::Deleted => (
                                                        rgba(0xef444415),
                                                        "-",
                                                        rgb(0xef4444),
                                                    ),
                                                    DiffLineType::Modified => (
                                                        rgba(0xf59e0b15),
                                                        "~",
                                                        rgb(0xf59e0b),
                                                    ),
                                                    DiffLineType::Unchanged => (
                                                        rgba(0x00000000),
                                                        " ",
                                                        theme.text_primary(),
                                                    ),
                                                };

                                                div()
                                                    .id(("diff-line", idx))
                                                    .bg(bg_color)
                                                    .flex()
                                                    .border_b_1()
                                                    .border_color(rgba(0x88888810))
                                                    // Left line num
                                                    .child(
                                                        div()
                                                            .w(px(40.0))
                                                            .px_1()
                                                            .py_0p5()
                                                            .text_right()
                                                            .text_color(theme.text_muted())
                                                            .bg(rgba(0x88888808))
                                                            .child(line.left_line_num.map(|n| n.to_string()).unwrap_or_default()),
                                                    )
                                                    // Left line text
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .px_2()
                                                            .py_0p5()
                                                            .overflow_hidden()
                                                            .text_color(if line.line_type == DiffLineType::Deleted || line.line_type == DiffLineType::Modified {
                                                                text_color
                                                            } else {
                                                                theme.text_primary()
                                                            })
                                                            .child(line.left_text.clone()),
                                                    )
                                                    // Change indicator
                                                    .child(
                                                        div()
                                                            .w(px(24.0))
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .text_color(text_color)
                                                            .font_weight(gpui::FontWeight::BOLD)
                                                            .child(indicator),
                                                    )
                                                    // Right line num
                                                    .child(
                                                        div()
                                                            .w(px(40.0))
                                                            .px_1()
                                                            .py_0p5()
                                                            .text_right()
                                                            .text_color(theme.text_muted())
                                                            .bg(rgba(0x88888808))
                                                            .child(line.right_line_num.map(|n| n.to_string()).unwrap_or_default()),
                                                    )
                                                    // Right line text
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .px_2()
                                                            .py_0p5()
                                                            .overflow_hidden()
                                                            .text_color(if line.line_type == DiffLineType::Added || line.line_type == DiffLineType::Modified {
                                                                text_color
                                                            } else {
                                                                theme.text_primary()
                                                            })
                                                            .child(line.right_text.clone()),
                                                    )
                                            })
                                        ),
                                )
                            }),
                    )
                    // 4. Footer & Actions
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_4()
                            .py_2p5()
                            .bg(theme.modal_header_bg())
                            .border_t_1()
                            .border_color(theme.border())
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_secondary())
                                    .child(if is_identical {
                                        "Žádné rozdíly".to_string()
                                    } else {
                                        format!("Rozdílů: {}", diff_count)
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("diff-btn-external")
                                            .cursor_pointer()
                                            .px_3()
                                            .py_1()
                                            .rounded(px(4.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .text_color(theme.text_primary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.open_external_diff();
                                                cx.notify();
                                            }))
                                            .child(t.btn_open_external_diff()),
                                    )
                                    .child(
                                        div()
                                            .id("diff-btn-close")
                                            .cursor_pointer()
                                            .px_4()
                                            .py_1()
                                            .rounded(px(4.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .text_color(theme.text_primary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.active_modal = ActiveModal::None;
                                                this.file_compare = None;
                                                cx.notify();
                                            }))
                                            .child(t.btn_cancel()),
                                    ),
                            ),
                    ),
            )
    }
}
