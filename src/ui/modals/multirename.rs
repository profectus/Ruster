use gpui::{div, prelude::*, px, rgb, rgba, Context, IntoElement, MouseButton};
use crate::app::RusterView;
use crate::icons::render_icon;
use crate::model::ActiveModal;
use crate::rename::{CaseTransform, ExtTransform};
use crate::ui::helpers::render_field_value_with_cursor;

impl RusterView {
    pub fn render_multirename_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_dark = theme.is_dark();

        let total_count = self.multirename_previews.len();
        let changed_count = self.multirename_previews.iter().filter(|p| p.has_change).count();
        let collision_count = self.multirename_previews.iter().filter(|p| p.has_collision).count();
        let can_rename = changed_count > 0 && collision_count == 0;

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
                    .w(px(780.0))
                    .h(px(600.0))
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
                                            .child(t.modal_multirename_title()),
                                    ),
                            )
                            .child(
                                div()
                                    .id("multirename-close-x")
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded(px(4.0))
                                    .hover(|s| s.bg(theme.popover_item_active()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.active_modal = ActiveModal::None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/close.svg", 14.0, theme.c_icon_secondary())),
                            ),
                    )
                    // 2. Controls & Configuration
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
                            // Row 1: Find & Replace inputs
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
                                                    .child(t.lbl_find()),
                                            )
                                            .child(
                                                div()
                                                    .id("multirename-field-find")
                                                    .cursor_pointer()
                                                    .px_2p5()
                                                    .py_1()
                                                    .rounded(px(5.0))
                                                    .bg(theme.input_bg())
                                                    .border_1()
                                                    .border_color(if self.multirename_active_field == 0 {
                                                        theme.input_border_focus()
                                                    } else {
                                                        theme.border()
                                                    })
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.multirename_active_field = 0;
                                                        this.multirename_cursor_pos = this.multirename_config.find_pattern.chars().count();
                                                        cx.notify();
                                                    }))
                                                    .child(render_field_value_with_cursor(
                                                        &self.multirename_config.find_pattern,
                                                        self.multirename_active_field == 0,
                                                        self.multirename_cursor_pos,
                                                        "[N]",
                                                        false,
                                                        is_dark,
                                                    )),
                                            ),
                                    )
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
                                                    .child(t.lbl_replace()),
                                            )
                                            .child(
                                                div()
                                                    .id("multirename-field-replace")
                                                    .cursor_pointer()
                                                    .px_2p5()
                                                    .py_1()
                                                    .rounded(px(5.0))
                                                    .bg(theme.input_bg())
                                                    .border_1()
                                                    .border_color(if self.multirename_active_field == 1 {
                                                        theme.input_border_focus()
                                                    } else {
                                                        theme.border()
                                                    })
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.multirename_active_field = 1;
                                                        this.multirename_cursor_pos = this.multirename_config.replace_pattern.chars().count();
                                                        cx.notify();
                                                    }))
                                                    .child(render_field_value_with_cursor(
                                                        &self.multirename_config.replace_pattern,
                                                        self.multirename_active_field == 1,
                                                        self.multirename_cursor_pos,
                                                        "",
                                                        false,
                                                        is_dark,
                                                    )),
                                            ),
                                    ),
                            )
                            // Row 2: Regex, Case sensitive, Placeholders Help
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
                                            .gap_3()
                                            // RegEx toggle
                                            .child(
                                                div()
                                                    .id("multirename-toggle-regex")
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.multirename_config.use_regex = !this.multirename_config.use_regex;
                                                        this.recompute_multirename_previews();
                                                        cx.notify();
                                                    }))
                                                    .child(
                                                        div()
                                                            .w(px(14.0))
                                                            .h(px(14.0))
                                                            .rounded(px(3.0))
                                                            .border_1()
                                                            .border_color(if self.multirename_config.use_regex { rgb(0x2563eb) } else { theme.border() })
                                                            .bg(if self.multirename_config.use_regex { rgb(0x2563eb) } else { theme.input_bg() })
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .when(self.multirename_config.use_regex, |d| {
                                                                d.child(render_icon("icons/check.svg", 10.0, 0xffffff))
                                                            }),
                                                    )
                                                    .child(div().text_color(theme.text_primary()).child(t.lbl_regex())),
                                            )
                                            // Case sensitive toggle
                                            .child(
                                                div()
                                                    .id("multirename-toggle-casesens")
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.multirename_config.case_sensitive = !this.multirename_config.case_sensitive;
                                                        this.recompute_multirename_previews();
                                                        cx.notify();
                                                    }))
                                                    .child(
                                                        div()
                                                            .w(px(14.0))
                                                            .h(px(14.0))
                                                            .rounded(px(3.0))
                                                            .border_1()
                                                            .border_color(if self.multirename_config.case_sensitive { rgb(0x2563eb) } else { theme.border() })
                                                            .bg(if self.multirename_config.case_sensitive { rgb(0x2563eb) } else { theme.input_bg() })
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .when(self.multirename_config.case_sensitive, |d| {
                                                                d.child(render_icon("icons/check.svg", 10.0, 0xffffff))
                                                            }),
                                                    )
                                                    .child(div().text_color(theme.text_primary()).child(t.lbl_case_sensitive())),
                                            ),
                                    )
                                    // Placeholders hint
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted())
                                            .child("[N] Název   [E] Přípona   [C] Počítadlo"),
                                    ),
                            )
                            // Row 3: Counter start, step, digits, Case transform, Ext transform
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
                                            .gap_3()
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(div().text_color(theme.text_secondary()).child(t.lbl_start()))
                                                    .child(
                                                        div()
                                                            .id("multirename-counter-start")
                                                            .cursor_pointer()
                                                            .px_2()
                                                            .py_0p5()
                                                            .rounded(px(4.0))
                                                            .bg(theme.input_bg())
                                                            .border_1()
                                                            .border_color(if self.multirename_active_field == 2 { theme.input_border_focus() } else { theme.border() })
                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                this.multirename_active_field = 2;
                                                                this.multirename_cursor_pos = this.multirename_config.counter_start.to_string().chars().count();
                                                                cx.notify();
                                                            }))
                                                            .child(render_field_value_with_cursor(
                                                                &self.multirename_config.counter_start.to_string(),
                                                                self.multirename_active_field == 2,
                                                                self.multirename_cursor_pos,
                                                                "1",
                                                                false,
                                                                is_dark,
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(div().text_color(theme.text_secondary()).child(t.lbl_step()))
                                                    .child(
                                                        div()
                                                            .id("multirename-counter-step")
                                                            .cursor_pointer()
                                                            .px_2()
                                                            .py_0p5()
                                                            .rounded(px(4.0))
                                                            .bg(theme.input_bg())
                                                            .border_1()
                                                            .border_color(if self.multirename_active_field == 3 { theme.input_border_focus() } else { theme.border() })
                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                this.multirename_active_field = 3;
                                                                this.multirename_cursor_pos = this.multirename_config.counter_step.to_string().chars().count();
                                                                cx.notify();
                                                            }))
                                                            .child(render_field_value_with_cursor(
                                                                &self.multirename_config.counter_step.to_string(),
                                                                self.multirename_active_field == 3,
                                                                self.multirename_cursor_pos,
                                                                "1",
                                                                false,
                                                                is_dark,
                                                            )),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(div().text_color(theme.text_secondary()).child(t.lbl_digits()))
                                                    .child(
                                                        div()
                                                            .id("multirename-counter-digits")
                                                            .cursor_pointer()
                                                            .px_2()
                                                            .py_0p5()
                                                            .rounded(px(4.0))
                                                            .bg(theme.input_bg())
                                                            .border_1()
                                                            .border_color(if self.multirename_active_field == 4 { theme.input_border_focus() } else { theme.border() })
                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                this.multirename_active_field = 4;
                                                                this.multirename_cursor_pos = this.multirename_config.counter_digits.to_string().chars().count();
                                                                cx.notify();
                                                            }))
                                                            .child(render_field_value_with_cursor(
                                                                &self.multirename_config.counter_digits.to_string(),
                                                                self.multirename_active_field == 4,
                                                                self.multirename_cursor_pos,
                                                                "2",
                                                                false,
                                                                is_dark,
                                                            )),
                                                    ),
                                            ),
                                    )
                                    // Transforms: Case & Extension
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .id("multirename-cycle-case")
                                                    .cursor_pointer()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded(px(4.0))
                                                    .bg(theme.button_bg())
                                                    .border_1()
                                                    .border_color(theme.border())
                                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.multirename_config.case_transform = match this.multirename_config.case_transform {
                                                            CaseTransform::None => CaseTransform::Lower,
                                                            CaseTransform::Lower => CaseTransform::Upper,
                                                            CaseTransform::Upper => CaseTransform::Title,
                                                            CaseTransform::Title => CaseTransform::None,
                                                        };
                                                        this.recompute_multirename_previews();
                                                        cx.notify();
                                                    }))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme.text_primary())
                                                            .child(match self.multirename_config.case_transform {
                                                                CaseTransform::None => "Case: —",
                                                                CaseTransform::Lower => "Case: abc",
                                                                CaseTransform::Upper => "Case: ABC",
                                                                CaseTransform::Title => "Case: Abc",
                                                            }),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .id("multirename-cycle-ext")
                                                    .cursor_pointer()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded(px(4.0))
                                                    .bg(theme.button_bg())
                                                    .border_1()
                                                    .border_color(theme.border())
                                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.multirename_config.ext_transform = match this.multirename_config.ext_transform {
                                                            ExtTransform::Keep => ExtTransform::Lower,
                                                            ExtTransform::Lower => ExtTransform::Upper,
                                                            ExtTransform::Upper => ExtTransform::Keep,
                                                        };
                                                        this.recompute_multirename_previews();
                                                        cx.notify();
                                                    }))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme.text_primary())
                                                            .child(match self.multirename_config.ext_transform {
                                                                ExtTransform::Keep => "Ext: .—",
                                                                ExtTransform::Lower => "Ext: .ext",
                                                                ExtTransform::Upper => "Ext: .EXT",
                                                            }),
                                                    ),
                                            ),
                                    ),
                            ),
                    )
                    // 3. Preview Table
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
                                    .child(div().w(px(320.0)).child(t.col_old_name()))
                                    .child(div().w(px(30.0)).flex().justify_center().child("→"))
                                    .child(div().flex_1().child(t.col_new_name()))
                                    .child(div().w(px(90.0)).text_right().child("Stav")),
                            )
                            // Table Rows
                            .child(
                                div()
                                    .id("multirename-preview-list")
                                    .flex_1()
                                    .overflow_y_scroll()
                                    .p_1()
                                    .flex()
                                    .flex_col()
                                    .children(
                                        self.multirename_previews.iter().enumerate().map(|(idx, preview)| {
                                            let is_even = idx % 2 == 0;
                                            let row_bg = if is_even { theme.pane_bg() } else { theme.modal_card_bg() };

                                            div()
                                                .id(("rename-prev", idx))
                                                .px_3()
                                                .py_1()
                                                .rounded(px(3.0))
                                                .bg(row_bg)
                                                .hover(|s| s.bg(theme.popover_item_active()))
                                                .flex()
                                                .items_center()
                                                .text_xs()
                                                .child(
                                                    div()
                                                        .w(px(320.0))
                                                        .overflow_hidden()
                                                        .text_color(theme.text_primary())
                                                        .child(preview.original_name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(30.0))
                                                        .flex()
                                                        .justify_center()
                                                        .text_color(theme.text_muted())
                                                        .child("→"),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .overflow_hidden()
                                                        .text_color(if preview.has_collision {
                                                            rgb(0xef4444)
                                                        } else if preview.has_change {
                                                            rgb(0x10b981)
                                                        } else {
                                                            theme.text_muted()
                                                        })
                                                        .font_weight(if preview.has_change {
                                                            gpui::FontWeight::MEDIUM
                                                        } else {
                                                            gpui::FontWeight::NORMAL
                                                        })
                                                        .child(preview.new_name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(90.0))
                                                        .flex()
                                                        .justify_end()
                                                        .child(
                                                            if preview.has_collision {
                                                                div()
                                                                    .px_1p5()
                                                                    .py_0p5()
                                                                    .rounded(px(3.0))
                                                                    .bg(rgba(0xef444420))
                                                                    .text_color(rgb(0xef4444))
                                                                    .text_xs()
                                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                                    .child("Kolize!")
                                                            } else if preview.has_change {
                                                                div()
                                                                    .px_1p5()
                                                                    .py_0p5()
                                                                    .rounded(px(3.0))
                                                                    .bg(rgba(0x10b98120))
                                                                    .text_color(rgb(0x10b981))
                                                                    .text_xs()
                                                                    .child("Změněno")
                                                            } else {
                                                                div()
                                                                    .text_color(theme.text_muted())
                                                                    .child("—")
                                                            }
                                                        ),
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
                                    .child(format!(
                                        "Celkem: {}  |  Změněno: {}{}",
                                        total_count,
                                        changed_count,
                                        if collision_count > 0 {
                                            format!("  |  Kolizí: {}", collision_count)
                                        } else {
                                            String::new()
                                        }
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("multirename-btn-cancel")
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
                                                this.active_modal = ActiveModal::None;
                                                cx.notify();
                                            }))
                                            .child(t.btn_cancel()),
                                    )
                                    .child(
                                        div()
                                            .id("multirename-btn-confirm")
                                            .cursor_pointer()
                                            .px_4()
                                            .py_1()
                                            .rounded(px(4.0))
                                            .bg(if can_rename { rgb(0x2563eb) } else { rgb(0x64748b) })
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(rgb(0xffffff))
                                            .when(can_rename, |d| {
                                                d.hover(|s| s.bg(rgb(0x1d4ed8)))
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.confirm_multirename(cx);
                                                    }))
                                            })
                                            .child(t.btn_rename()),
                                    ),
                            ),
                    ),
            )
    }
}
