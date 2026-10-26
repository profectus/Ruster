use gpui::{div, prelude::*, px, rgb, rgba, Context, IntoElement, MouseButton};
use crate::app::MacScpView;
use crate::icons::render_icon;
use crate::model::{format_bytes, ActiveModal, SyncAction, SyncCriterion, SyncDirection};

impl MacScpView {
    pub fn render_sync_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let state = &self.sync_state;

        let remote_display = self
            .sessions
            .get(self.active_session_idx)
            .map(|s| format!("{}:{}", s.host, s.current_path))
            .unwrap_or_else(|| self.right_dir.to_string_lossy().to_string());

        let local_display = self.local_dir.to_string_lossy().to_string();

        let total_items = state.items.len();
        let selected_count = state.selected_count();
        let selected_bytes = state.selected_bytes();

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
                            .py_3()
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
                                    .child(render_icon("icons/sync.svg", 16.0, theme.c_icon_active()))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(t.modal_sync_title()),
                                    ),
                            )
                            .child(
                                div()
                                    .id("sync-modal-close-x")
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded(px(4.0))
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.active_modal = ActiveModal::None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/close.svg", 13.0, theme.c_icon_default())),
                            ),
                    )
                    // 2. Configuration Card
                    .child(
                        div()
                            .flex_shrink_0()
                            .p_3p5()
                            .border_b_1()
                            .border_color(theme.border())
                            .bg(theme.modal_card_bg())
                            .flex()
                            .flex_col()
                            .gap_3()
                            // Direction selection segmented buttons
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(self.render_direction_button(
                                        "sync-dir-l2r",
                                        t.sync_dir_local_to_remote(),
                                        "icons/arrow-right.svg",
                                        state.direction == SyncDirection::LocalToRemote,
                                        SyncDirection::LocalToRemote,
                                        cx,
                                    ))
                                    .child(self.render_direction_button(
                                        "sync-dir-r2l",
                                        t.sync_dir_remote_to_local(),
                                        "icons/arrow-left.svg",
                                        state.direction == SyncDirection::RemoteToLocal,
                                        SyncDirection::RemoteToLocal,
                                        cx,
                                    ))
                                    .child(self.render_direction_button(
                                        "sync-dir-twoway",
                                        t.sync_dir_two_way(),
                                        "icons/arrow-left-right.svg",
                                        state.direction == SyncDirection::TwoWay,
                                        SyncDirection::TwoWay,
                                        cx,
                                    )),
                            )
                            // Folder paths cards
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex_1()
                                            .p_2()
                                            .rounded(px(6.0))
                                            .bg(theme.input_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(render_icon("icons/hard-drive.svg", 14.0, theme.c_icon_default()))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .flex()
                                                    .flex_col()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .text_color(theme.text_secondary())
                                                            .child(t.sync_col_source()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme.text_primary())
                                                            .line_clamp(1)
                                                            .child(local_display),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .child(render_icon(
                                                match state.direction {
                                                    SyncDirection::LocalToRemote => "icons/arrow-right.svg",
                                                    SyncDirection::RemoteToLocal => "icons/arrow-left.svg",
                                                    SyncDirection::TwoWay => "icons/arrow-left-right.svg",
                                                },
                                                16.0,
                                                theme.c_icon_active(),
                                            )),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .p_2()
                                            .rounded(px(6.0))
                                            .bg(theme.input_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(render_icon("icons/server.svg", 14.0, theme.c_icon_default()))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .flex()
                                                    .flex_col()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .text_color(theme.text_secondary())
                                                            .child(t.sync_col_target()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme.text_primary())
                                                            .line_clamp(1)
                                                            .child(remote_display),
                                                    ),
                                            ),
                                    ),
                            )
                            // Options and Compare button row
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_wrap()
                                            .items_center()
                                            .gap_x_3p5()
                                            .gap_y_1p5()
                                            // Checkbox: recursive
                                            .child(self.render_sync_checkbox(
                                                "sync-chk-recursive",
                                                t.sync_opt_recursive(),
                                                state.recursive,
                                                cx,
                                                |this, cx| {
                                                    this.sync_state.recursive = !this.sync_state.recursive;
                                                    this.run_sync_comparison(cx);
                                                },
                                            ))
                                            // Checkbox: time & size
                                            .child(self.render_sync_checkbox(
                                                "sync-chk-time-size",
                                                t.sync_opt_time_size(),
                                                state.criterion == SyncCriterion::TimeAndSize,
                                                cx,
                                                |this, cx| {
                                                    this.sync_state.criterion = if this.sync_state.criterion == SyncCriterion::TimeAndSize {
                                                        SyncCriterion::TimeOnly
                                                    } else {
                                                        SyncCriterion::TimeAndSize
                                                    };
                                                    this.run_sync_comparison(cx);
                                                },
                                            ))
                                            // Checkbox: mirror delete
                                            .child(self.render_sync_checkbox(
                                                "sync-chk-mirror",
                                                t.sync_opt_mirror(),
                                                state.mirror_delete,
                                                cx,
                                                |this, cx| {
                                                    this.sync_state.mirror_delete = !this.sync_state.mirror_delete;
                                                    this.run_sync_comparison(cx);
                                                },
                                            )),
                                    )
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .child(
                                                div()
                                                    .id("sync-btn-compare")
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .px_3()
                                                    .py_1p5()
                                                    .rounded(px(5.0))
                                                    .bg(theme.button_bg())
                                                    .border_1()
                                                    .border_color(theme.border_strong())
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(theme.text_primary())
                                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.run_sync_comparison(cx);
                                                        }),
                                                    )
                                                    .child(render_icon("icons/sync.svg", 12.0, theme.c_icon_default()))
                                                    .child(t.sync_btn_compare()),
                                            ),
                                    ),
                            ),
                    )
                    // 3. Results Preview Area
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .bg(theme.pane_bg())
                            .child(self.render_sync_results_area(cx)),
                    )
                    // 4. Footer
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_4()
                            .py_3()
                            .rounded_b(px(10.0))
                            .border_t_1()
                            .border_color(theme.border())
                            .bg(theme.modal_header_bg())
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_secondary())
                                    .child(if state.has_compared && !state.items.is_empty() {
                                        format!(
                                            "Vybráno: {} / {} souborů ({})",
                                            selected_count,
                                            total_items,
                                            format_bytes(selected_bytes)
                                        )
                                    } else {
                                        String::new()
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("sync-btn-cancel")
                                            .cursor_pointer()
                                            .px_3p5()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .text_color(theme.text_primary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.active_modal = ActiveModal::None;
                                                    cx.notify();
                                                }),
                                            )
                                            .child(t.btn_cancel()),
                                    )
                                    .child(
                                        div()
                                            .id("sync-btn-execute")
                                            .cursor_pointer()
                                            .flex()
                                            .items_center()
                                            .gap_1p5()
                                            .px_4()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(if selected_count > 0 {
                                                rgb(0x2563eb)
                                            } else {
                                                rgba(0x2563eb66)
                                            })
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xffffff))
                                            .when(selected_count > 0, |el| {
                                                el.hover(|s| s.bg(rgb(0x1d4ed8)))
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.execute_synchronization(cx);
                                                        }),
                                                    )
                                            })
                                            .child(render_icon("icons/sync.svg", 12.0, 0xffffff))
                                            .child(t.sync_btn_start()),
                                    ),
                            ),
                    ),
            )
    }

    fn render_direction_button(
        &self,
        id: &'static str,
        label: &'static str,
        icon: &'static str,
        active: bool,
        dir: SyncDirection,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = self.theme();
        div()
            .id(id)
            .cursor_pointer()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .gap_1p5()
            .py_1p5()
            .px_2()
            .rounded(px(6.0))
            .bg(if active { rgb(0x2563eb) } else { theme.button_bg() })
            .border_1()
            .border_color(if active { rgb(0x1d4ed8) } else { theme.border() })
            .text_xs()
            .font_weight(if active {
                gpui::FontWeight::SEMIBOLD
            } else {
                gpui::FontWeight::NORMAL
            })
            .text_color(if active { rgb(0xffffff) } else { theme.text_primary() })
            .when(!active, |el| el.hover(|s| s.bg(theme.button_hover_bg())))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if this.sync_state.direction != dir {
                        this.sync_state.direction = dir;
                        this.run_sync_comparison(cx);
                    }
                }),
            )
            .child(render_icon(icon, 13.0, if active { 0xffffff } else { theme.c_icon_default() }))
            .child(label)
    }

    fn render_sync_checkbox(
        &self,
        id: &'static str,
        label: &'static str,
        checked: bool,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let theme = self.theme();
        div()
            .id(id)
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2()
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                on_click(this, cx);
            }))
            .child(
                div()
                    .w(px(15.0))
                    .h(px(15.0))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(if checked { rgb(0x2563eb) } else { theme.border_strong() })
                    .bg(if checked { rgb(0x2563eb) } else { theme.input_bg() })
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(checked, |el| el.child(render_icon("icons/check.svg", 10.0, 0xffffff))),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_primary())
                    .child(label),
            )
    }

    fn render_sync_results_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let state = &self.sync_state;

        if state.is_comparing {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .child(render_icon("icons/sync.svg", 26.0, theme.c_icon_active()))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.text_primary())
                        .child(t.sync_status_comparing()),
                )
                .when_some(state.progress_msg.clone(), |el, pmsg| {
                    el.child(
                        div()
                            .max_w(px(550.0))
                            .px_3()
                            .py_1p5()
                            .rounded(px(5.0))
                            .bg(theme.input_bg())
                            .border_1()
                            .border_color(theme.border())
                            .text_xs()
                            .text_color(theme.text_secondary())
                            .truncate()
                            .child(pmsg),
                    )
                });
        }

        if let Some(err) = &state.error_message {
            return div()
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2p5()
                .p_4()
                .child(render_icon("icons/alert-circle.svg", 26.0, 0xef4444))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(0xef4444))
                        .child("Chyba při porovnávání adresářů"),
                )
                .child(
                    div()
                        .id("sync-error-scroll")
                        .max_h(px(140.0))
                        .max_w(px(600.0))
                        .overflow_y_scroll()
                        .p_2p5()
                        .rounded(px(6.0))
                        .bg(theme.input_bg())
                        .border_1()
                        .border_color(theme.border())
                        .text_xs()
                        .text_color(theme.text_secondary())
                        .child(err.clone()),
                );
        }

        if state.has_compared && state.items.is_empty() {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .child(render_icon("icons/check.svg", 28.0, 0x16a34a))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(rgb(0x16a34a))
                        .child(t.sync_status_identical()),
                );
        }

        if !state.has_compared {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .child(render_icon("icons/sync.svg", 24.0, theme.c_icon_default()))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_secondary())
                        .child("Vyberte směr synchronizace a spusťte porovnání adresářů."),
                );
        }

        let all_selected = state.all_selected();

        div()
            .size_full()
            .flex()
            .flex_col()
            // Table Sub-header toolbar
            .child(
                div()
                    .px_3()
                    .py_1p5()
                    .bg(theme.modal_header_bg())
                    .border_b_1()
                    .border_color(theme.border())
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.text_secondary())
                            .child(format!("K přenosu nalezeno {} položek", state.items.len())),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .id("sync-select-all")
                                    .cursor_pointer()
                                    .px_2()
                                    .py_0p5()
                                    .rounded(px(3.0))
                                    .bg(theme.button_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .text_xs()
                                    .text_color(theme.text_primary())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.sync_state.set_all_selected(true);
                                            cx.notify();
                                        }),
                                    )
                                    .child(t.sync_select_all()),
                            )
                            .child(
                                div()
                                    .id("sync-deselect-all")
                                    .cursor_pointer()
                                    .px_2()
                                    .py_0p5()
                                    .rounded(px(3.0))
                                    .bg(theme.button_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .text_xs()
                                    .text_color(theme.text_primary())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.sync_state.set_all_selected(false);
                                            cx.notify();
                                        }),
                                    )
                                    .child(t.sync_deselect_all()),
                            ),
                    ),
            )
            // Table Columns Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_3()
                    .py_2()
                    .bg(theme.table_header_bg())
                    .border_b_1()
                    .border_color(theme.border())
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.text_secondary())
                    .child(
                        div()
                            .w(px(26.0))
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .cursor_pointer()
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .rounded(px(3.0))
                                    .border_1()
                                    .border_color(if all_selected { rgb(0x2563eb) } else { theme.border_strong() })
                                    .bg(if all_selected { rgb(0x2563eb) } else { theme.input_bg() })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when(all_selected, |el| el.child(render_icon("icons/check.svg", 10.0, 0xffffff)))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            let cur = this.sync_state.all_selected();
                                            this.sync_state.set_all_selected(!cur);
                                            cx.notify();
                                        }),
                                    ),
                            ),
                    )
                    .child(div().w(px(90.0)).child(t.sync_col_action()))
                    .child(div().flex_1().child(t.sync_col_file()))
                    .child(div().w(px(85.0)).child(t.col_size()))
                    .child(div().w(px(160.0)).child(t.sync_col_reason())),
            )
            // Table Rows
            .child(
                div()
                    .id("sync-table-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .children(state.items.iter().map(|item| {
                        let item_id = item.id;
                        let is_selected = item.selected;
                        let action = item.action;
                        let size_text = if item.is_dir {
                            "složka".to_string()
                        } else {
                            match action {
                                SyncAction::Upload => format_bytes(item.local_size),
                                SyncAction::Download => format_bytes(item.remote_size),
                                _ => format_bytes(item.local_size.max(item.remote_size)),
                            }
                        };

                        let (badge_bg, badge_fg, action_label) = match action {
                            SyncAction::Upload => (
                                rgba(0x16a34a22),
                                rgb(0x16a34a),
                                "Nahrát ➔",
                            ),
                            SyncAction::Download => (
                                rgba(0x2563eb22),
                                rgb(0x2563eb),
                                "⬅ Stáhnout",
                            ),
                            SyncAction::DeleteRemote => (
                                rgba(0xdc262622),
                                rgb(0xdc2626),
                                "Smazat",
                            ),
                            SyncAction::DeleteLocal => (
                                rgba(0xdc262622),
                                rgb(0xdc2626),
                                "Smazat",
                            ),
                        };

                        div()
                            .id(gpui::ElementId::NamedInteger("sync-item".into(), item_id as u64))
                            .flex()
                            .items_center()
                            .px_3()
                            .py_1p5()
                            .border_b_1()
                            .border_color(theme.border())
                            .hover(|s| s.bg(theme.button_hover_bg()))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.sync_state.toggle_item(item_id);
                                    cx.notify();
                                }),
                            )
                            // Checkbox
                            .child(
                                div()
                                    .w(px(26.0))
                                    .flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .w(px(14.0))
                                            .h(px(14.0))
                                            .rounded(px(3.0))
                                            .border_1()
                                            .border_color(if is_selected { rgb(0x2563eb) } else { theme.border_strong() })
                                            .bg(if is_selected { rgb(0x2563eb) } else { theme.input_bg() })
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .when(is_selected, |el| el.child(render_icon("icons/check.svg", 10.0, 0xffffff))),
                                    ),
                            )
                            // Action Badge
                            .child(
                                div()
                                    .w(px(90.0))
                                    .flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .px_1p5()
                                            .py_0p5()
                                            .rounded(px(4.0))
                                            .bg(badge_bg)
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(badge_fg)
                                            .child(action_label),
                                    ),
                            )
                            // File Path
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .overflow_hidden()
                                    .child(render_icon(
                                        if item.is_dir { "icons/folder.svg" } else { "icons/file.svg" },
                                        13.0,
                                        if item.is_dir { 0x2563eb } else { theme.c_icon_default() },
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_primary())
                                            .line_clamp(1)
                                            .child(item.rel_path.clone()),
                                    ),
                            )
                            // Size
                            .child(
                                div()
                                    .w(px(85.0))
                                    .text_xs()
                                    .text_color(theme.text_secondary())
                                    .child(size_text),
                            )
                            // Reason
                            .child(
                                div()
                                    .w(px(160.0))
                                    .text_xs()
                                    .text_color(theme.text_secondary())
                                    .line_clamp(1)
                                    .child(item.reason.clone()),
                            )
                    })),
            )
    }
}
