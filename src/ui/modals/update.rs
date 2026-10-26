use gpui::{
    div, prelude::*, px, rgb, Context, IntoElement, MouseButton,
};
use crate::app::MacScpView;
use crate::icons::render_icon;
use crate::model::ActiveModal;
use crate::update::UpdateStatus;

impl MacScpView {
    pub fn render_update_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let t = self.i18n();

        div()
            .id("modal-update-backdrop")
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
                    .w(px(480.0))
                    .rounded(px(10.0))
                    .bg(theme.modal_bg())
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_xl()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    // Header
                    .child(
                        div()
                            .px_5()
                            .py_3p5()
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
                                            .child(t.update_title()),
                                    ),
                            )
                            .child(
                                div()
                                    .id("btn-close-update-x")
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded(px(4.0))
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.active_modal = ActiveModal::None;
                                        cx.notify();
                                    }))
                                    .child(render_icon("icons/close.svg", 13.0, theme.c_icon_default())),
                            ),
                    )
                    // Body
                    .child(
                        div()
                            .id("modal-update-body")
                            .p_5()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(match &self.update_status {
                                Some(UpdateStatus::Checking) | None => {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .py_8()
                                        .gap_3()
                                        .child(render_icon("icons/sync.svg", 28.0, theme.c_icon_active()))
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(theme.text_secondary())
                                                .child(t.update_checking_msg()),
                                        )
                                }
                                Some(UpdateStatus::UpToDate(ver)) => {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .py_6()
                                        .gap_3()
                                        .child(render_icon("icons/check.svg", 32.0, 0x16a34a))
                                        .child(
                                            div()
                                                .text_base()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(theme.text_primary())
                                                .child(t.update_uptodate_msg()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.text_muted())
                                                .child(format!("MacSCP v{ver}")),
                                        )
                                }
                                Some(UpdateStatus::Available(info)) => {
                                    let latest = info.latest_version.clone();
                                    let current = info.current_version.clone();
                                    let copied = self.update_copied_notice;

                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_3p5()
                                        .child(
                                            div()
                                                .text_base()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(theme.text_primary())
                                                .child(t.update_available_msg()),
                                        )
                                        // Versions card
                                        .child(
                                            div()
                                                .px_3()
                                                .py_2()
                                                .rounded(px(6.0))
                                                .bg(theme.modal_card_bg())
                                                .border_1()
                                                .border_color(theme.border())
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme.text_secondary())
                                                        .child(format!("Installed: v{current}")),
                                                )
                                                .child(
                                                    div()
                                                        .px_2()
                                                        .py_0p5()
                                                        .rounded(px(4.0))
                                                        .bg(rgb(0x2563eb))
                                                        .text_xs()
                                                        .font_weight(gpui::FontWeight::BOLD)
                                                        .text_color(rgb(0xffffff))
                                                        .child(format!("Latest: v{latest}")),
                                                ),
                                        )
                                        // Homebrew Box
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_1p5()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(gpui::FontWeight::MEDIUM)
                                                        .text_color(theme.text_secondary())
                                                        .child(t.update_brew_hint()),
                                                )
                                                .child(
                                                    div()
                                                        .id("brew-upgrade-box")
                                                        .px_3()
                                                        .py_2()
                                                        .rounded(px(5.0))
                                                        .bg(theme.input_bg())
                                                        .border_1()
                                                        .border_color(theme.input_border())
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .child(
                                                            div()
                                                                .font_family(".AppleSystemUIFontMonospaced")
                                                                .text_xs()
                                                                .text_color(theme.text_primary())
                                                                .child("brew update && brew upgrade --cask macscp"),
                                                        )
                                                        .child(
                                                            div()
                                                                .id("btn-copy-brew-cmd")
                                                                .cursor_pointer()
                                                                .px_2()
                                                                .py_1()
                                                                .rounded(px(4.0))
                                                                .bg(if copied {
                                                                    rgb(0x16a34a)
                                                                } else {
                                                                    theme.button_bg()
                                                                })
                                                                .border_1()
                                                                .border_color(theme.border())
                                                                .hover(|s| s.bg(theme.button_hover_bg()))
                                                                .text_xs()
                                                                .text_color(if copied {
                                                                    rgb(0xffffff)
                                                                } else {
                                                                    theme.text_primary()
                                                                })
                                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                    this.copy_brew_command();
                                                                    cx.notify();
                                                                }))
                                                                .child(if copied {
                                                                    t.update_copied()
                                                                } else {
                                                                    t.update_copy_cmd()
                                                                }),
                                                        ),
                                                ),
                                        )
                                }
                                Some(UpdateStatus::Updating { version, message }) => {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .py_8()
                                        .gap_3()
                                        .child(render_icon("icons/sync.svg", 32.0, 0x2563eb))
                                        .child(
                                            div()
                                                .text_base()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(theme.text_primary())
                                                .child(t.update_updating_title()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.text_secondary())
                                                .child(message.clone()),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded(px(4.0))
                                                .bg(theme.modal_card_bg())
                                                .text_xs()
                                                .text_color(theme.text_muted())
                                                .child(format!("v{version}")),
                                        )
                                }
                                Some(UpdateStatus::ReadyToRestart { version }) => {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .py_6()
                                        .gap_3()
                                        .child(render_icon("icons/check.svg", 36.0, 0x16a34a))
                                        .child(
                                            div()
                                                .text_base()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(theme.text_primary())
                                                .child(t.update_success_title()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.text_secondary())
                                                .child(t.update_restart_desc()),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded(px(4.0))
                                                .bg(rgb(0x16a34a))
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(rgb(0xffffff))
                                                .child(format!("v{version}")),
                                        )
                                }
                                Some(UpdateStatus::Error(err)) => {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .py_6()
                                        .gap_3()
                                        .child(render_icon("icons/alert-circle.svg", 32.0, 0xef4444))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme.text_primary())
                                                .child("Chyba při ověřování aktualizací"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.text_muted())
                                                .child(err.clone()),
                                        )
                                }
                            }),
                    )
                    // Footer
                    .child(
                        div()
                            .px_5()
                            .py_3()
                            .rounded_b(px(10.0))
                            .border_t_1()
                            .border_color(theme.border())
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_2()
                            .bg(theme.modal_header_bg())
                            .children(match &self.update_status {
                                Some(UpdateStatus::Available(info)) => {
                                    let latest = info.latest_version.clone();
                                    let url = info.release_url.clone();
                                    vec![
                                        div()
                                            .id("btn-close-update")
                                            .cursor_pointer()
                                            .px_4()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_primary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.active_modal = ActiveModal::None;
                                                cx.notify();
                                            }))
                                            .child(t.btn_cancel())
                                            .into_any_element(),
                                        div()
                                            .id("btn-github-update")
                                            .cursor_pointer()
                                            .px_3p5()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_primary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                let _ = std::process::Command::new("open").arg(&url).spawn();
                                                this.active_modal = ActiveModal::None;
                                                cx.notify();
                                            }))
                                            .child(t.update_download_btn())
                                            .into_any_element(),
                                        div()
                                            .id("btn-start-update-now")
                                            .cursor_pointer()
                                            .px_4()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(rgb(0x2563eb))
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(0xffffff))
                                            .hover(|s| s.bg(rgb(0x1d4ed8)))
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                this.start_in_app_update(latest.clone(), cx);
                                            }))
                                            .child(t.update_btn_now())
                                            .into_any_element(),
                                    ]
                                }
                                Some(UpdateStatus::Updating { .. }) => {
                                    vec![
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_muted())
                                            .child("Probíhá instalace...")
                                            .into_any_element(),
                                    ]
                                }
                                Some(UpdateStatus::ReadyToRestart { .. }) => {
                                    vec![
                                        div()
                                            .id("btn-restart-later")
                                            .cursor_pointer()
                                            .px_4()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_primary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.active_modal = ActiveModal::None;
                                                cx.notify();
                                            }))
                                            .child(t.update_later_btn())
                                            .into_any_element(),
                                        div()
                                            .id("btn-restart-now")
                                            .cursor_pointer()
                                            .px_4()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(rgb(0x16a34a))
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(0xffffff))
                                            .hover(|s| s.bg(rgb(0x15803d)))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, _| {
                                                crate::update::restart_app();
                                            }))
                                            .child(t.update_restart_btn())
                                            .into_any_element(),
                                    ]
                                }
                                _ => {
                                    vec![
                                        div()
                                            .id("btn-close-update")
                                            .cursor_pointer()
                                            .px_4()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.button_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_primary())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.active_modal = ActiveModal::None;
                                                cx.notify();
                                            }))
                                            .child("OK")
                                            .into_any_element(),
                                    ]
                                }
                            }),
                    ),
            )
    }
}
