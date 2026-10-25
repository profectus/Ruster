use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};
use crate::app::MacScpView;
use crate::model::{ActiveModal, ActivePane};
use crate::ui::helpers::render_field_value_with_cursor;

impl MacScpView {
    pub fn render_text_input_modal(
        &self,
        title: &'static str,
        confirm_label: &'static str,
        is_new_folder: bool,
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
                    .w(px(380.0))
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
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary())
                            .child(title),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded(px(5.0))
                            .bg(theme.input_bg())
                            .border_1()
                            .border_color(theme.input_border_focus())
                            .text_xs()
                            .text_color(theme.text_primary())
                            .child(render_field_value_with_cursor(
                                &self.modal_input_text,
                                true,
                                self.modal_cursor_pos,
                                "",
                                false,
                                is_dark,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                div()
                                    .id("text-modal-cancel")
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
                                    .id("text-modal-confirm")
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
                                        cx.listener(move |this, _, _, cx| {
                                            if is_new_folder {
                                                this.confirm_new_folder();
                                            } else {
                                                this.confirm_rename();
                                            }
                                            cx.notify();
                                        }),
                                    )
                                    .child(confirm_label),
                            ),
                    ),
            )
    }

    pub fn render_delete_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let selected_count = match self.active_pane {
            ActivePane::Local => self
                .local_entries
                .iter()
                .filter(|e| e.is_any_selected() && e.name != "..")
                .count(),
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    self.right_entries
                        .iter()
                        .filter(|e| e.is_any_selected() && e.name != "..")
                        .count()
                } else {
                    self.sessions
                        .get(self.active_session_idx)
                        .map(|s| {
                            s.entries
                                .iter()
                                .filter(|e| e.is_any_selected() && e.name != "..")
                                .count()
                        })
                        .unwrap_or(0)
                }
            }
        };

        let confirm_message = if selected_count > 1 {
            format!("Delete {} selected items permanently?", selected_count)
        } else {
            let target_name = match self.active_pane {
                ActivePane::Local => self
                    .local_entries
                    .get(self.local_selected)
                    .map(|e| e.name.clone())
                    .unwrap_or_default(),
                ActivePane::Remote => {
                    if self.sessions.is_empty() {
                        self.right_entries
                            .get(self.right_selected)
                            .map(|e| e.name.clone())
                            .unwrap_or_default()
                    } else {
                        self.sessions
                            .get(self.active_session_idx)
                            .and_then(|s| s.entries.get(s.selected_index))
                            .map(|e| e.name.clone())
                            .unwrap_or_default()
                    }
                }
            };
            format!("Delete '{}' permanently?", target_name)
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
                    .w(px(380.0))
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
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text_primary())
                            .child(t.modal_delete_title()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_secondary())
                            .child(confirm_message),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                div()
                                    .id("del-modal-cancel")
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
                                    .id("del-modal-confirm")
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(4.0))
                                    .bg(rgb(0xdc2626))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(0xffffff))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.confirm_delete();
                                            cx.notify();
                                        }),
                                    )
                                    .child(t.btn_delete()),
                            ),
                    ),
            )
    }
}
