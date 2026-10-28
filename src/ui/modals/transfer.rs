use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};
use crate::app::RusterView;
use crate::icons::render_icon;
use crate::model::{format_bytes, ActiveModal, TransferDirection};

impl RusterView {
    pub fn render_transfer_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let state = self.active_transfer.as_ref();
        let (dir_badge, dir_color) = match state.map(|s| s.direction).unwrap_or(TransferDirection::Download) {
            TransferDirection::Download => ("FTP ➔ Místní", 0x16a34a),
            TransferDirection::Upload => ("Místní ➔ FTP", 0x2563eb),
        };

        let current_file = state.map(|s| s.current_filename.as_str()).unwrap_or("Příprava...");
        let total = state.map(|s| s.total_items).unwrap_or(1);
        let current_idx = state.map(|s| s.current_item_index).unwrap_or(1);
        let is_completed = state.map(|s| s.is_completed).unwrap_or(false);
        let error_msg = state.and_then(|s| s.error_message.as_ref());
        let src_path = state.map(|s| s.source_path.as_str()).unwrap_or("");
        let tgt_path = state.map(|s| s.target_path.as_str()).unwrap_or("");

        let file_transferred = state.map(|s| s.file_bytes_transferred).unwrap_or(0);
        let file_total = state.map(|s| s.file_bytes_total).unwrap_or(0);
        let file_pct = state.map(|s| s.file_percent()).unwrap_or(0.0);
        let overall_pct = state.map(|s| s.overall_percent()).unwrap_or(0.0);
        let speed_mbps = state.map(|s| s.speed_mbps).unwrap_or(0.0);
        let eta_seconds = state.and_then(|s| s.eta_seconds);

        let speed_text = if speed_mbps > 0.05 {
            format!("{:.1} MB/s", speed_mbps)
        } else if is_completed {
            "Dokončeno".to_string()
        } else {
            "-- MB/s".to_string()
        };

        let eta_text = match eta_seconds {
            Some(secs) if secs < 3600 => format!("{:02}:{:02}", secs / 60, secs % 60),
            Some(secs) => format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60),
            None => "--:--".to_string(),
        };

        let file_progress_detail = if file_total > 0 {
            format!("{} / {} ({:.0}%)", format_bytes(file_transferred), format_bytes(file_total), file_pct)
        } else if file_transferred > 0 {
            format!("{} ({:.0}%)", format_bytes(file_transferred), file_pct)
        } else {
            format!("{:.0}%", file_pct)
        };

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
                    .w(px(500.0))
                    .rounded(px(10.0))
                    .bg(theme.modal_bg())
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_lg()
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(render_icon("icons/arrow-left-right.svg", 16.0, if is_completed { 0x16a34a } else { theme.c_icon_active() }))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(if is_completed {
                                                "Přenos dokončen"
                                            } else {
                                                "Probíhá přenos souborů..."
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded(px(4.0))
                                    .bg(theme.modal_card_bg())
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(dir_color))
                                    .child(dir_badge),
                            ),
                    )
                    // Current item info & file progress
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .text_xs()
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(current_file.to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_color(theme.text_muted())
                                            .child(file_progress_detail),
                                    ),
                            )
                            // File Progress bar
                            .child(
                                div()
                                    .w_full()
                                    .h(px(9.0))
                                    .rounded(px(4.5))
                                    .bg(theme.progress_bg())
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .h_full()
                                            .w(gpui::DefiniteLength::Fraction(file_pct / 100.0))
                                            .rounded(px(4.5))
                                            .bg(if is_completed { rgb(0x16a34a) } else { theme.progress_bar() }),
                                    ),
                            )
                            // Transfer stats: speed & ETA
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .text_xs()
                                    .text_color(theme.text_secondary())
                                    .child(
                                        div()
                                            .flex()
                                            .gap_1()
                                            .child("Rychlost:")
                                            .child(div().font_weight(gpui::FontWeight::MEDIUM).text_color(theme.text_primary()).child(speed_text)),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .gap_1()
                                            .child("Zbývá:")
                                            .child(div().font_weight(gpui::FontWeight::MEDIUM).text_color(theme.text_primary()).child(eta_text)),
                                    ),
                            ),
                    )
                    // Overall progress (if multiple items)
                    .when(total > 1, |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .pt_1()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .text_xs()
                                        .child(
                                            div()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme.text_primary())
                                                .child(format!("Celkem: položka {} z {}", current_idx, total)),
                                        )
                                        .child(
                                            div()
                                                .text_color(theme.text_secondary())
                                                .child(format!("{:.0}%", overall_pct)),
                                        ),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .h(px(6.0))
                                        .rounded(px(3.0))
                                        .bg(theme.progress_bg())
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .h_full()
                                                .w(gpui::DefiniteLength::Fraction(overall_pct / 100.0))
                                                .rounded(px(3.0))
                                                .bg(if is_completed { rgb(0x16a34a) } else { theme.progress_bar() }),
                                        ),
                                ),
                        )
                    })
                    // Paths detail
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .p_2p5()
                            .rounded(px(6.0))
                            .bg(theme.modal_card_bg())
                            .border_1()
                            .border_color(theme.border())
                            .text_xs()
                            .text_color(theme.text_secondary())
                            .child(
                                div()
                                    .flex()
                                    .gap_1p5()
                                    .child(div().font_weight(gpui::FontWeight::MEDIUM).child("Zdroj:"))
                                    .child(div().text_color(theme.text_primary()).child(src_path.to_string())),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_1p5()
                                    .child(div().font_weight(gpui::FontWeight::MEDIUM).child("Cíl:"))
                                    .child(div().text_color(theme.text_primary()).child(tgt_path.to_string())),
                            ),
                    )
                    // Error message if any
                    .when_some(error_msg, |el, err| {
                        el.child(
                            div()
                                .px_3()
                                .py_2()
                                .rounded(px(6.0))
                                .bg(if theme.is_dark() { rgb(0x451a1a) } else { rgb(0xfef2f2) })
                                .border_1()
                                .border_color(if theme.is_dark() { rgb(0x7f1d1d) } else { rgb(0xfecaca) })
                                .text_xs()
                                .text_color(if theme.is_dark() { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                                .child(format!("Chyba přenosu: {}", err)),
                        )
                    })
                    // Footer buttons
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .pt_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted())
                                    .child(if is_completed {
                                        "Všechny soubory byly přeneseny."
                                    } else {
                                        "Přenos běží v odděleném procesu."
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .cursor_pointer()
                                            .px_3()
                                            .py_1p5()
                                            .rounded(px(6.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_primary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.active_modal = ActiveModal::None;
                                                    cx.notify();
                                                }),
                                            )
                                            .child(if is_completed { "Zavřít" } else { "Běžet na pozadí" }),
                                    ),
                            ),
                    ),
            )
    }

    pub fn render_transfer_error_modal(
        &self,
        prompt: crate::model::TransferErrorPromptState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_dark = theme.is_dark();

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
                    .w(px(520.0))
                    .rounded(px(10.0))
                    .bg(theme.modal_bg())
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_lg()
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_3p5()
                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(render_icon("icons/alert-circle.svg", 20.0, 0xef4444))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text_primary())
                                    .child(t.modal_transfer_error_title()),
                            ),
                    )
                    // Error details box
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .p_3()
                            .rounded(px(6.0))
                            .bg(if is_dark { rgb(0x2d1515) } else { rgb(0xfef2f2) })
                            .border_1()
                            .border_color(if is_dark { rgb(0x7f1d1d) } else { rgb(0xfecaca) })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(format!("Soubor: {}", prompt.filename)),
                                    ),
                            )
                            .when(!prompt.source.is_empty(), |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.text_secondary())
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .whitespace_nowrap()
                                        .child(format!("Zdroj: {}", prompt.source)),
                                )
                            })
                            .when(!prompt.target.is_empty(), |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.text_secondary())
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .whitespace_nowrap()
                                        .child(format!("Cíl: {}", prompt.target)),
                                )
                            })
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(if is_dark { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                                    .child(format!("Chyba: {}", prompt.error)),
                            ),
                    )
                    // Question text
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.text_secondary())
                            .child(t.transfer_error_question()),
                    )
                    // Action buttons
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .pt_1()
                            // Left side: Cancel entire transfer
                            .child(
                                div()
                                    .id("btn-transfer-err-cancel")
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1p5()
                                    .rounded(px(5.0))
                                    .bg(if is_dark { rgb(0x451a1a) } else { rgb(0xfee2e2) })
                                    .border_1()
                                    .border_color(if is_dark { rgb(0x7f1d1d) } else { rgb(0xfca5a5) })
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(if is_dark { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                                    .hover(|s| s.bg(if is_dark { rgb(0x5a2323) } else { rgb(0xfecaca) }))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.handle_transfer_error_action(crate::model::TransferErrorAction::Cancel, cx);
                                        }),
                                    )
                                    .child(t.btn_abort_transfer()),
                            )
                            // Right side: Skip all, Skip, Retry
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("btn-transfer-err-skip-all")
                                            .cursor_pointer()
                                            .px_2p5()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_secondary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.handle_transfer_error_action(crate::model::TransferErrorAction::SkipAll, cx);
                                                }),
                                            )
                                            .child(t.btn_skip_all()),
                                    )
                                    .child(
                                        div()
                                            .id("btn-transfer-err-skip")
                                            .cursor_pointer()
                                            .px_2p5()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_primary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.handle_transfer_error_action(crate::model::TransferErrorAction::Skip, cx);
                                                }),
                                            )
                                            .child(t.btn_skip()),
                                    )
                                    .child(
                                        div()
                                            .id("btn-transfer-err-retry")
                                            .cursor_pointer()
                                            .px_3p5()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(rgb(0x2563eb))
                                            .hover(|s| s.bg(rgb(0x1d4ed8)))
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xffffff))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.handle_transfer_error_action(crate::model::TransferErrorAction::Retry, cx);
                                                }),
                                            )
                                            .child(t.btn_retry()),
                                    ),
                            ),
                    ),
            )
    }
}
