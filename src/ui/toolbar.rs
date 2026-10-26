use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};
use crate::app::MacScpView;
use crate::icons::render_icon;
use crate::model::{ActiveModal, ActivePane};

impl MacScpView {
    pub fn render_top_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let both_trees_active = self.show_local_tree && self.show_remote_tree;
        let has_session = !self.sessions.is_empty();

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(38.0))
            .px_3()
            .bg(theme.top_bar_bg())
            .border_b_1()
            .border_color(theme.border())
            // Left: Connection dropdown / Connect button + Icon-only Connection Manager button + Theme toggle
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .when(has_session, |el| {
                        let active_session = match self.sessions.get(self.active_session_idx) {
                            Some(s) => s,
                            None => return el,
                        };
                        let active_label = format!("{}@{}", active_session.username, active_session.host);
                        let active_proto = active_session.protocol.short_badge();
                        let active_color = active_session.env_color;

                        el.child(
                            div()
                                .id("session-dropdown-btn")
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .gap_2()
                                .h(px(26.0))
                                .px_2p5()
                                .rounded(px(5.0))
                                .bg(theme.button_bg())
                                .border_1()
                                .border_color(if self.session_dropdown_open {
                                    theme.border_focused()
                                } else {
                                    theme.border_strong()
                                })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.session_dropdown_open = !this.session_dropdown_open;
                                        this.context_menu = None;
                                        cx.notify();
                                    }),
                                )
                                .child(
                                    div()
                                        .w(px(6.0))
                                        .h(px(6.0))
                                        .rounded(px(3.0))
                                        .bg(rgb(active_color)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.text_primary())
                                        .child(active_label),
                                )
                                .child(
                                    div()
                                        .px_1()
                                        .rounded(px(3.0))
                                        .bg(theme.button_secondary_bg())
                                        .text_xs()
                                        .text_color(theme.text_secondary())
                                        .child(active_proto),
                                )
                                .child(render_icon("icons/chevron-down.svg", 13.0, theme.c_icon_default())),
                        )
                        .child(
                            div()
                                .id("btn-disconnect-active")
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .gap_1()
                                .h(px(26.0))
                                .px_2()
                                .rounded(px(5.0))
                                .bg(if theme.is_dark() { rgb(0x451a1a) } else { rgb(0xfee2e2) })
                                .border_1()
                                .border_color(if theme.is_dark() { rgb(0x7f1d1d) } else { rgb(0xfca5a5) })
                                .text_xs()
                                .text_color(if theme.is_dark() { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.disconnect_active_session();
                                        cx.notify();
                                    }),
                                )
                                .child(render_icon("icons/close.svg", 11.0, if theme.is_dark() { 0xfca5a5 } else { 0xb91c1c }))
                                .child(t.btn_disconnect()),
                        )
                    })
                    .when(!has_session, |el| {
                        el.child(
                            div()
                                .id("btn-quick-connect")
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .h(px(26.0))
                                .px_2p5()
                                .rounded(px(5.0))
                                .bg(theme.button_bg())
                                .border_1()
                                .border_color(theme.border_strong())
                                .text_xs()
                                .text_color(theme.text_primary())
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .hover(|s| s.bg(theme.button_hover_bg()))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.open_site_manager();
                                        cx.notify();
                                    }),
                                )
                                .child(render_icon("icons/server.svg", 13.0, theme.c_icon_active()))
                                .child(t.btn_connect_server()),
                        )
                    })
                    .child(self.render_icon_btn(
                        "btn-open-site-manager",
                        "icons/server.svg",
                        self.active_modal == ActiveModal::SiteManager,
                        cx,
                        |this, cx| {
                            this.open_site_manager();
                            cx.notify();
                        },
                    ))
                    .child(self.render_icon_btn(
                        "btn-open-settings",
                        "icons/settings.svg",
                        self.active_modal == ActiveModal::Settings,
                        cx,
                        |this, cx| {
                            this.active_modal = ActiveModal::Settings;
                            cx.notify();
                        },
                    ))
                    .child(self.render_icon_btn(
                        "btn-toggle-theme",
                        if theme.is_dark() {
                            "icons/sun.svg"
                        } else {
                            "icons/moon.svg"
                        },
                        false,
                        cx,
                        |this, cx| {
                            this.toggle_theme(cx);
                        },
                    )),
            )
            // Right: Layout & quick filter icon buttons
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(self.render_icon_btn(
                        "act-quick-filter",
                        "icons/search.svg",
                        self.show_local_filter || self.show_remote_filter,
                        cx,
                        |this, cx| {
                            let should_show = !(this.show_local_filter && this.show_remote_filter);
                            this.show_local_filter = should_show;
                            this.show_remote_filter = should_show;
                            if !should_show {
                                this.local_filter_query.clear();
                                this.remote_filter_query.clear();
                            }
                            this.ensure_local_selection();
                            this.ensure_remote_selection();
                            cx.notify();
                        },
                    ))
                    .child(self.render_icon_btn(
                        "act-local-pane",
                        "icons/columns-2.svg",
                        self.show_local_pane,
                        cx,
                        |this, cx| {
                            this.show_local_pane = !this.show_local_pane;
                            if !this.show_local_pane {
                                this.active_pane = ActivePane::Remote;
                            }
                            this.persist_app_state();
                            cx.notify();
                        },
                    ))
                    .when(self.show_local_pane, |el| {
                        el.child(self.render_icon_btn(
                            "act-swap-panes",
                            "icons/arrow-left-right.svg",
                            self.swap_panes,
                            cx,
                            |this, cx| {
                                this.swap_panes = !this.swap_panes;
                                cx.notify();
                            },
                        ))
                    })
                    .child(self.render_icon_btn(
                        "act-tree-both",
                        "icons/folder-tree.svg",
                        both_trees_active,
                        cx,
                        |this, cx| {
                            let next = !(this.show_local_tree && this.show_remote_tree);
                            this.show_local_tree = next;
                            this.show_remote_tree = next;
                            cx.notify();
                        },
                    )),
            )
    }

    pub fn render_icon_btn(
        &self,
        id: &'static str,
        icon_path: &'static str,
        active: bool,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let theme = self.theme();
        let icon_color = if active {
            theme.c_icon_active()
        } else {
            theme.c_icon_dark()
        };

        div()
            .id(id)
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .w(px(26.0))
            .h(px(26.0))
            .rounded(px(5.0))
            .bg(if active {
                theme.button_active_bg()
            } else {
                theme.button_bg()
            })
            .border_1()
            .border_color(if active {
                theme.button_active_border()
            } else {
                theme.border()
            })
            .hover(|s| s.bg(theme.button_hover_bg()))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    action(this, cx);
                }),
            )
            .child(render_icon(icon_path, 14.0, icon_color))
    }

    pub fn render_pane_filter_bar(&self, pane: ActivePane, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_active_pane = self.active_pane == pane;
        let (query, visible_count, total_count, close_id) = match pane {
            ActivePane::Local => {
                let vc = self
                    .filtered_local_indices()
                    .iter()
                    .filter(|&&i| self.local_entries.get(i).map(|e| e.name != "..").unwrap_or(false))
                    .count();
                let tc = self.local_entries.iter().filter(|e| e.name != "..").count();
                (self.local_filter_query.clone(), vc, tc, "close-local-filter-bar")
            }
            ActivePane::Remote => {
                let (vc, tc) = if self.sessions.is_empty() {
                    let v = self
                        .filtered_remote_indices()
                        .iter()
                        .filter(|&&i| self.right_entries.get(i).map(|e| e.name != "..").unwrap_or(false))
                        .count();
                    let t = self.right_entries.iter().filter(|e| e.name != "..").count();
                    (v, t)
                } else if let Some(s) = self.sessions.get(self.active_session_idx) {
                    let v = self
                        .filtered_remote_indices()
                        .iter()
                        .filter(|&&i| s.entries.get(i).map(|e| e.name != "..").unwrap_or(false))
                        .count();
                    let t = s.entries.iter().filter(|e| e.name != "..").count();
                    (v, t)
                } else {
                    (0, 0)
                };
                (self.remote_filter_query.clone(), vc, tc, "close-remote-filter-bar")
            }
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(30.0))
            .px_2p5()
            .bg(theme.filter_bar_bg())
            .border_b_1()
            .border_color(theme.border())
            .text_xs()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if this.active_pane != pane {
                        this.active_pane = pane;
                        cx.notify();
                    }
                }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(render_icon(
                        "icons/filter.svg",
                        13.0,
                        if is_active_pane {
                            theme.c_icon_active()
                        } else {
                            theme.c_icon_secondary()
                        },
                    ))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .px_2()
                            .py_0p5()
                            .rounded(px(4.0))
                            .bg(theme.input_bg())
                            .border_1()
                            .border_color(if is_active_pane {
                                theme.border_focused()
                            } else {
                                theme.border()
                            })
                            .child(
                                if query.is_empty() {
                                    div()
                                        .text_color(theme.text_muted())
                                        .child(t.filter_placeholder())
                                } else {
                                    div()
                                        .text_color(theme.text_primary())
                                        .child(query.clone())
                                },
                            )
                            .when(is_active_pane, |el| {
                                el.child(
                                    div()
                                        .w(px(1.5))
                                        .h(px(12.0))
                                        .bg(theme.text_accent())
                                        .ml_0p5(),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_color(theme.text_muted())
                            .child(format!("{}/{}", visible_count, total_count)),
                    ),
            )
            .child(
                div()
                    .id(close_id)
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded(px(4.0))
                    .hover(|s| s.bg(theme.button_hover_bg()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            match pane {
                                ActivePane::Local => {
                                    this.show_local_filter = false;
                                    this.local_filter_query.clear();
                                    this.ensure_local_selection();
                                }
                                ActivePane::Remote => {
                                    this.show_remote_filter = false;
                                    this.remote_filter_query.clear();
                                    this.ensure_remote_selection();
                                }
                            }
                            cx.notify();
                        }),
                    )
                    .child(render_icon("icons/close.svg", 12.0, theme.c_icon_default())),
            )
    }
}
