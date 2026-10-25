use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};
use crate::app::MacScpView;
use crate::icons::render_icon;
use crate::model::ActiveModal;

impl MacScpView {
    pub fn render_sync_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();

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
                    .w(px(480.0))
                    .rounded(px(8.0))
                    .bg(theme.modal_bg())
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_lg()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(render_icon("icons/sync.svg", 14.0, theme.c_icon_dark()))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text_primary())
                                    .child(t.modal_sync_title()),
                            ),
                    )
                    .child(
                        div()
                            .p_3()
                            .rounded(px(6.0))
                            .bg(theme.modal_card_bg())
                            .border_1()
                            .border_color(theme.border())
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .text_xs()
                            .text_color(theme.text_secondary())
                            .child(format!("Local: {}", self.local_dir.to_string_lossy()))
                            .child(format!(
                                "Remote: {}",
                                self.sessions
                                    .get(self.active_session_idx)
                                    .map(|s| s.current_path.as_str())
                                    .unwrap_or_else(|| self.right_dir.to_str().unwrap_or("/"))
                            ))
                            .child("Mode: Mirror changes by modification time and size"),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                div()
                                    .id("sync-close")
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
                                    .id("sync-run")
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(4.0))
                                    .bg(rgb(0x2563eb))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(0xffffff))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.active_modal = ActiveModal::None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(t.btn_start()),
                            ),
                    ),
            )
    }
}
