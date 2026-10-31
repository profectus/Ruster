use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};
use crate::app::RusterView;
use crate::icons::render_icon;
use crate::model::{format_bytes, ActiveModal};
use crate::ui::helpers::render_field_value_with_cursor;

impl RusterView {
    pub fn render_find_files_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_dark = theme.is_dark();

        let count = self.find_results.len();

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
                    .w(px(800.0))
                    .h(px(620.0))
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
                                    .child(render_icon("icons/search.svg", 16.0, theme.c_icon_active()))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(t.modal_find_title()),
                                    ),
                            )
                            .child(
                                div()
                                    .id("find-close-x")
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded(px(4.0))
                                    .hover(|s| s.bg(theme.popover_item_active()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.stop_find_files();
                                            this.active_modal = ActiveModal::None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/close.svg", 14.0, theme.c_icon_secondary())),
                            ),
                    )
                    // 2. Query Form
                    .child(
                        div()
                            .flex_shrink_0()
                            .p_3()
                            .border_b_1()
                            .border_color(theme.border())
                            .bg(theme.modal_card_bg())
                            .flex()
                            .flex_col()
                            .gap_2()
                            // Row 1: Search in directory & File mask
                            .child(
                                div()
                                    .flex()
                                    .gap_3()
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(theme.text_secondary())
                                                    .child(t.lbl_search_in()),
                                            )
                                            .child(
                                                div()
                                                    .id("find-field-root")
                                                    .cursor_pointer()
                                                    .px_2p5()
                                                    .py_1()
                                                    .rounded(px(5.0))
                                                    .bg(theme.input_bg())
                                                    .border_1()
                                                    .border_color(if self.find_active_field == 0 {
                                                        theme.input_border_focus()
                                                    } else {
                                                        theme.border()
                                                    })
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.find_active_field = 0;
                                                        this.find_cursor_pos = this.find_query.root_dir.to_string_lossy().chars().count();
                                                        cx.notify();
                                                    }))
                                                    .child(render_field_value_with_cursor(
                                                        &self.find_query.root_dir.to_string_lossy(),
                                                        self.find_active_field == 0,
                                                        self.find_cursor_pos,
                                                        ".",
                                                        false,
                                                        is_dark,
                                                    )),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .w(px(240.0))
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(theme.text_secondary())
                                                    .child(t.lbl_file_mask()),
                                            )
                                            .child(
                                                div()
                                                    .id("find-field-mask")
                                                    .cursor_pointer()
                                                    .px_2p5()
                                                    .py_1()
                                                    .rounded(px(5.0))
                                                    .bg(theme.input_bg())
                                                    .border_1()
                                                    .border_color(if self.find_active_field == 1 {
                                                        theme.input_border_focus()
                                                    } else {
                                                        theme.border()
                                                    })
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.find_active_field = 1;
                                                        this.find_cursor_pos = this.find_query.name_pattern.chars().count();
                                                        cx.notify();
                                                    }))
                                                    .child(render_field_value_with_cursor(
                                                        &self.find_query.name_pattern,
                                                        self.find_active_field == 1,
                                                        self.find_cursor_pos,
                                                        "*",
                                                        false,
                                                        is_dark,
                                                    )),
                                            ),
                                    ),
                            )
                            // Row 2: Containing text
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_secondary())
                                            .child(t.lbl_containing_text()),
                                    )
                                    .child(
                                        div()
                                            .id("find-field-text")
                                            .cursor_pointer()
                                            .px_2p5()
                                            .py_1()
                                            .rounded(px(5.0))
                                            .bg(theme.input_bg())
                                            .border_1()
                                            .border_color(if self.find_active_field == 2 {
                                                theme.input_border_focus()
                                            } else {
                                                theme.border()
                                            })
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.find_active_field = 2;
                                                this.find_cursor_pos = this.find_query.containing_text.chars().count();
                                                cx.notify();
                                            }))
                                            .child(render_field_value_with_cursor(
                                                &self.find_query.containing_text,
                                                self.find_active_field == 2,
                                                self.find_cursor_pos,
                                                "",
                                                false,
                                                is_dark,
                                            )),
                                    ),
                            )
                            // Row 3: Checkboxes & Search / Stop Button
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .text_xs()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_4()
                                            // Recursive
                                            .child(
                                                div()
                                                    .id("find-toggle-recursive")
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.find_query.recursive = !this.find_query.recursive;
                                                        cx.notify();
                                                    }))
                                                    .child(
                                                        div()
                                                            .w(px(14.0))
                                                            .h(px(14.0))
                                                            .rounded(px(3.0))
                                                            .border_1()
                                                            .border_color(if self.find_query.recursive { rgb(0x2563eb) } else { theme.border() })
                                                            .bg(if self.find_query.recursive { rgb(0x2563eb) } else { theme.input_bg() })
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .when(self.find_query.recursive, |d| {
                                                                d.child(render_icon("icons/check.svg", 10.0, 0xffffff))
                                                            }),
                                                    )
                                                    .child(div().text_color(theme.text_primary()).child(t.lbl_recursive())),
                                            )
                                            // Case sensitive
                                            .child(
                                                div()
                                                    .id("find-toggle-casesens")
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.find_query.case_sensitive = !this.find_query.case_sensitive;
                                                        cx.notify();
                                                    }))
                                                    .child(
                                                        div()
                                                            .w(px(14.0))
                                                            .h(px(14.0))
                                                            .rounded(px(3.0))
                                                            .border_1()
                                                            .border_color(if self.find_query.case_sensitive { rgb(0x2563eb) } else { theme.border() })
                                                            .bg(if self.find_query.case_sensitive { rgb(0x2563eb) } else { theme.input_bg() })
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .when(self.find_query.case_sensitive, |d| {
                                                                d.child(render_icon("icons/check.svg", 10.0, 0xffffff))
                                                            }),
                                                    )
                                                    .child(div().text_color(theme.text_primary()).child(t.lbl_case_sensitive())),
                                            )
                                            // RegEx
                                            .child(
                                                div()
                                                    .id("find-toggle-regex")
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.find_query.use_regex = !this.find_query.use_regex;
                                                        cx.notify();
                                                    }))
                                                    .child(
                                                        div()
                                                            .w(px(14.0))
                                                            .h(px(14.0))
                                                            .rounded(px(3.0))
                                                            .border_1()
                                                            .border_color(if self.find_query.use_regex { rgb(0x2563eb) } else { theme.border() })
                                                            .bg(if self.find_query.use_regex { rgb(0x2563eb) } else { theme.input_bg() })
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .when(self.find_query.use_regex, |d| {
                                                                d.child(render_icon("icons/check.svg", 10.0, 0xffffff))
                                                            }),
                                                    )
                                                    .child(div().text_color(theme.text_primary()).child(t.lbl_regex())),
                                            ),
                                    )
                                    // Search / Stop Button
                                    .child(
                                        if self.find_is_searching {
                                            div()
                                                .id("find-btn-stop")
                                                .cursor_pointer()
                                                .px_3()
                                                .py_1()
                                                .rounded(px(4.0))
                                                .bg(rgb(0xd97706))
                                                .hover(|s| s.bg(rgb(0xb45309)))
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(rgb(0xffffff))
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                    this.stop_find_files();
                                                    cx.notify();
                                                }))
                                                .child(t.btn_stop())
                                        } else {
                                            div()
                                                .id("find-btn-start")
                                                .cursor_pointer()
                                                .px_3()
                                                .py_1()
                                                .rounded(px(4.0))
                                                .bg(rgb(0x2563eb))
                                                .hover(|s| s.bg(rgb(0x1d4ed8)))
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(rgb(0xffffff))
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                    this.start_find_files(cx);
                                                }))
                                                .child(t.btn_search())
                                        }
                                    ),
                            ),
                    )
                    // 3. Results Table
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            // Table Header
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .px_4()
                                    .py_1p5()
                                    .bg(theme.table_header_bg())
                                    .border_b_1()
                                    .border_color(theme.border())
                                    .flex()
                                    .items_center()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text_secondary())
                                    .child(div().w(px(220.0)).child("Název"))
                                    .child(div().flex_1().child("Cesta"))
                                    .child(div().w(px(80.0)).text_right().child("Velikost"))
                                    .child(div().w(px(130.0)).text_right().child("Změněno")),
                            )
                            // Table Rows
                            .child(
                                div()
                                    .id("find-results-list")
                                    .flex_1()
                                    .overflow_y_scroll()
                                    .p_1()
                                    .flex()
                                    .flex_col()
                                    .children(
                                        self.find_results.iter().enumerate().map(|(idx, res)| {
                                            let is_selected = idx == self.find_selected_idx;
                                            let is_even = idx % 2 == 0;
                                            let row_bg = if is_selected {
                                                theme.row_bg_selected()
                                            } else if is_even {
                                                theme.pane_bg()
                                            } else {
                                                theme.modal_card_bg()
                                            };

                                            div()
                                                .id(("find-res", idx))
                                                .px_3()
                                                .py_1()
                                                .rounded(px(3.0))
                                                .bg(row_bg)
                                                .hover(|s| s.bg(theme.popover_item_active()))
                                                .cursor_pointer()
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    this.find_selected_idx = idx;
                                                    cx.notify();
                                                }))
                                                .flex()
                                                .items_center()
                                                .text_xs()
                                                .child(
                                                    div()
                                                        .w(px(220.0))
                                                        .overflow_hidden()
                                                        .font_weight(if is_selected { gpui::FontWeight::SEMIBOLD } else { gpui::FontWeight::NORMAL })
                                                        .text_color(if is_selected { theme.row_text_selected() } else { theme.text_primary() })
                                                        .child(res.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .overflow_hidden()
                                                        .text_color(theme.text_muted())
                                                        .child(res.path.to_string_lossy().to_string()),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(80.0))
                                                        .text_right()
                                                        .text_color(theme.text_secondary())
                                                        .child(if res.is_dir { "<DIR>".to_string() } else { format_bytes(res.size) }),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(130.0))
                                                        .text_right()
                                                        .text_color(theme.text_muted())
                                                        .child(res.modified_str.clone()),
                                                )
                                        })
                                    ),
                            ),
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
                                    .child(if self.find_is_searching {
                                        format!("Hledám... Nalezeno: {}", count)
                                    } else {
                                        format!("Nalezeno položek: {}", count)
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("find-btn-close")
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
                                                this.stop_find_files();
                                                this.active_modal = ActiveModal::None;
                                                cx.notify();
                                            }))
                                            .child(t.btn_cancel()),
                                    )
                                    .child(
                                        div()
                                            .id("find-btn-feed")
                                            .cursor_pointer()
                                            .px_4()
                                            .py_1()
                                            .rounded(px(4.0))
                                            .bg(if count > 0 { rgb(0x2563eb) } else { rgb(0x64748b) })
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(rgb(0xffffff))
                                            .when(count > 0, |d| {
                                                d.hover(|s| s.bg(rgb(0x1d4ed8)))
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.feed_search_to_listbox(cx);
                                                    }))
                                            })
                                            .child(t.btn_feed_to_listbox()),
                                    ),
                            ),
                    ),
            )
    }
}
