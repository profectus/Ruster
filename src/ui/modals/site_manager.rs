use gpui::{div, prelude::*, px, rgb, rgba, Context, IntoElement, MouseButton, MouseDownEvent};
use crate::app::MacScpView;
use crate::config::{import_from_ssh_config, save_password_to_keychain, save_sites_to_disk};
use crate::icons::render_icon;
use crate::model::{ActiveModal, Protocol, SavedSiteProfile, SiteEditField, ENV_COLORS};
use crate::ui::helpers::render_field_value_with_cursor;

impl MacScpView {
    pub fn render_site_manager_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let fallback_site = SavedSiteProfile {
            name: String::new(),
            folder: String::new(),
            protocol: Protocol::Sftp,
            host: String::new(),
            port: 22,
            username: String::new(),
            password: None,
            auth_type: String::new(),
            env_color: 0x2563eb,
            max_connections: 2,
        };
        let selected_site = self
            .saved_sites
            .get(self.selected_site_idx)
            .unwrap_or(&fallback_site);
        let active_field = self.active_site_field;
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
                    .w(px(740.0))
                    .rounded(px(8.0))
                    .bg(theme.modal_bg())
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_4()
                            .py_2p5()
                            .rounded_t(px(8.0))
                            .bg(theme.modal_header_bg())
                            .border_b_1()
                            .border_color(theme.border())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(render_icon("icons/server.svg", 14.0, theme.c_icon_dark()))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(t.modal_site_manager_title()),
                                    ),
                            )
                            .child(
                                div()
                                    .id("close-site-modal")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.sort_saved_sites();
                                            let _ = save_sites_to_disk(&this.saved_sites);
                                            this.active_modal = ActiveModal::None;
                                            this.active_site_field = SiteEditField::None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/close.svg", 14.0, theme.c_icon_default())),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .h(px(460.0))
                            .child(
                                div()
                                    .w(px(230.0))
                                    .h_full()
                                    .rounded_bl(px(8.0))
                                    .bg(theme.modal_sidebar_bg())
                                    .border_r_1()
                                    .border_color(theme.border())
                                    .p_2()
                                    .flex()
                                    .flex_col()
                                    .justify_between()
                                    .child(
                                        div()
                                            .id("site-list-scroll")
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .overflow_y_scroll()
                                            .children(self.saved_sites.iter().enumerate().map(
                                                |(idx, site)| {
                                                    let is_sel = idx == self.selected_site_idx;
                                                    let proto = site.protocol;
                                                    div()
                                                        .id(("site-item", idx))
                                                        .cursor_pointer()
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .px_2p5()
                                                        .py_1p5()
                                                        .rounded(px(5.0))
                                                        .bg(if is_sel {
                                                            theme.row_bg_selected()
                                                        } else {
                                                            theme.modal_sidebar_bg()
                                                        })
                                                        .hover(|s| {
                                                            if !is_sel {
                                                                s.bg(theme.button_hover_bg())
                                                            } else {
                                                                s
                                                            }
                                                        })
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                                                                this.selected_site_idx = idx;
                                                                this.draft_protocol = proto;
                                                                this.active_site_field =
                                                                    SiteEditField::Name;
                                                                this.site_password_input = this
                                                                    .saved_sites
                                                                    .get(idx)
                                                                    .and_then(|s| s.password.clone())
                                                                    .unwrap_or_default();
                                                                this.site_cursor_pos =
                                                                    this.current_site_field_val().chars().count();
                                                                this.site_error_message = None;
                                                                if event.click_count >= 2 {
                                                                    this.connect_selected_site();
                                                                }
                                                                cx.notify();
                                                            }),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_1p5()
                                                                .overflow_hidden()
                                                                .child(
                                                                    div()
                                                                        .w(px(7.0))
                                                                        .h(px(7.0))
                                                                        .rounded(px(3.5))
                                                                        .flex_shrink_0()
                                                                        .bg(rgb(site.env_color)),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .overflow_hidden()
                                                                        .text_ellipsis()
                                                                        .whitespace_nowrap()
                                                                        .font_weight(
                                                                            gpui::FontWeight::MEDIUM,
                                                                        )
                                                                        .text_color(if is_sel {
                                                                            theme.row_text_selected()
                                                                        } else {
                                                                            theme.text_primary()
                                                                        })
                                                                        .child(site.display_name().to_string()),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(if is_sel {
                                                                    rgb(0xdbeafe)
                                                                } else {
                                                                    theme.text_secondary()
                                                                })
                                                                .child(site.protocol.short_badge()),
                                                        )
                                                },
                                            )),
                                    )
                                    .child(
                                        div()
                                            .pt_2()
                                            .border_t_1()
                                            .border_color(theme.border())
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .id("btn-new-site")
                                                    .cursor_pointer()
                                                    .px_2()
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
                                                            this.saved_sites.push(
                                                                SavedSiteProfile {
                                                                    name: String::new(),
                                                                    folder: String::new(),
                                                                    protocol: Protocol::Ftp,
                                                                    host: String::new(),
                                                                    port: 21,
                                                                    username: String::new(),
                                                                    password: None,
                                                                    auth_type: String::new(),
                                                                    env_color: 0x2563eb,
                                                                    max_connections: 2,
                                                                },
                                                            );
                                                            this.selected_site_idx =
                                                                this.saved_sites.len() - 1;
                                                            this.draft_protocol = Protocol::Ftp;
                                                            this.active_site_field =
                                                                SiteEditField::Name;
                                                            this.site_cursor_pos = 0;
                                                            this.site_password_input.clear();
                                                            this.site_error_message = None;
                                                            this.sort_saved_sites();
                                                            let _ = save_sites_to_disk(
                                                                &this.saved_sites,
                                                            );
                                                            cx.notify();
                                                        }),
                                                    )
                                                    .child("+ New"),
                                            )
                                            .child(
                                                div()
                                                    .id("btn-import-ssh")
                                                    .cursor_pointer()
                                                    .px_2()
                                                    .py_1()
                                                    .rounded(px(4.0))
                                                    .bg(theme.button_bg())
                                                    .border_1()
                                                    .border_color(theme.border())
                                                    .text_xs()
                                                    .text_color(theme.text_accent())
                                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            for s in import_from_ssh_config() {
                                                                if !this.saved_sites.iter().any(
                                                                    |existing| {
                                                                        existing.host == s.host
                                                                            && existing.username
                                                                                == s.username
                                                                    },
                                                                ) {
                                                                    this.saved_sites.push(s);
                                                                }
                                                            }
                                                            this.sort_saved_sites();
                                                            let _ = save_sites_to_disk(
                                                                &this.saved_sites,
                                                            );
                                                            cx.notify();
                                                        }),
                                                    )
                                                    .child("SSH Config"),
                                            )
                                            .child(
                                                div()
                                                    .id("btn-del-site")
                                                    .cursor_pointer()
                                                    .px_2()
                                                    .py_1()
                                                    .rounded(px(4.0))
                                                    .bg(theme.button_bg())
                                                    .border_1()
                                                    .border_color(theme.border())
                                                    .text_xs()
                                                    .text_color(rgb(0xdc2626))
                                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            if this.saved_sites.len() > 1 {
                                                                this.saved_sites
                                                                    .remove(this.selected_site_idx);
                                                                if this.selected_site_idx
                                                                    >= this.saved_sites.len()
                                                                {
                                                                    this.selected_site_idx =
                                                                        this.saved_sites.len() - 1;
                                                                }
                                                                let _ = save_sites_to_disk(
                                                                    &this.saved_sites,
                                                                );
                                                                cx.notify();
                                                            }
                                                        }),
                                                    )
                                                    .child("Del"),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .p_4()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .id("site-form-scroll")
                                            .flex_1()
                                            .min_h_0()
                                            .overflow_y_scroll()
                                            .pr_1()
                                            .flex()
                                            .flex_col()
                                            .gap_2p5()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(theme.text_secondary())
                                                    .child(t.label_protocol()),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_1()
                                                    .children(
                                                        [
                                                            Protocol::Sftp,
                                                            Protocol::Scp,
                                                            Protocol::FtpsExplicit,
                                                            Protocol::FtpsImplicit,
                                                            Protocol::Ftp,
                                                        ]
                                                        .into_iter()
                                                        .enumerate()
                                                        .map(|(i, proto)| {
                                                            let active =
                                                                self.draft_protocol == proto;
                                                            div()
                                                                .id(("proto-btn", i))
                                                                .cursor_pointer()
                                                                .px_2p5()
                                                                .py_1()
                                                                .rounded(px(4.0))
                                                                .bg(if active {
                                                                    theme.button_active_bg()
                                                                } else {
                                                                    theme.button_secondary_bg()
                                                                })
                                                                .border_1()
                                                                .border_color(if active {
                                                                    theme.button_active_border()
                                                                } else {
                                                                    theme.border()
                                                                })
                                                                .text_xs()
                                                                .text_color(if active {
                                                                    theme.text_accent()
                                                                } else {
                                                                    theme.text_secondary()
                                                                })
                                                                .hover(|s| s.bg(theme.button_hover_bg()))
                                                                .on_mouse_down(
                                                                    MouseButton::Left,
                                                                    cx.listener(
                                                                        move |this, _, _, cx| {
                                                                            this.draft_protocol =
                                                                                proto;
                                                                            if let Some(s) = this
                                                                                .saved_sites
                                                                                .get_mut(
                                                                                this.selected_site_idx,
                                                                            ) {
                                                                                s.protocol = proto;
                                                                                s.port = proto
                                                                                    .default_port();
                                                                            }
                                                                            cx.notify();
                                                                        },
                                                                    ),
                                                                )
                                                                .child(proto.label())
                                                        }),
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
                                                    .child(
                                                        div()
                                                            .id("edit-site-name")
                                                            .cursor_pointer()
                                                            .flex()
                                                            .items_center()
                                                            .justify_between()
                                                            .px_2()
                                                            .py_1()
                                                            .rounded(px(4.0))
                                                            .bg(if active_field
                                                                == SiteEditField::Name
                                                            {
                                                                theme.button_active_bg()
                                                            } else {
                                                                theme.modal_card_bg()
                                                            })
                                                            .on_mouse_down(
                                                                MouseButton::Left,
                                                                cx.listener(|this, _, _, cx| {
                                                                    this.active_site_field =
                                                                        SiteEditField::Name;
                                                                    this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                    cx.notify();
                                                                }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.col_name()),
                                                            )
                                                            .child(render_field_value_with_cursor(
                                                                &selected_site.name,
                                                                active_field == SiteEditField::Name,
                                                                self.site_cursor_pos,
                                                                "My Connection",
                                                                false,
                                                                is_dark,
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("edit-site-host")
                                                            .cursor_pointer()
                                                            .flex()
                                                            .items_center()
                                                            .justify_between()
                                                            .px_2()
                                                            .py_1()
                                                            .rounded(px(4.0))
                                                            .bg(if active_field
                                                                == SiteEditField::Host
                                                            {
                                                                theme.button_active_bg()
                                                            } else {
                                                                theme.modal_card_bg()
                                                            })
                                                            .on_mouse_down(
                                                                MouseButton::Left,
                                                                cx.listener(|this, _, _, cx| {
                                                                    this.active_site_field =
                                                                        SiteEditField::Host;
                                                                    this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                    cx.notify();
                                                                }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.label_host()),
                                                            )
                                                            .child(render_field_value_with_cursor(
                                                                &selected_site.host,
                                                                active_field == SiteEditField::Host,
                                                                self.site_cursor_pos,
                                                                "ftp.example.com",
                                                                false,
                                                                is_dark,
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("edit-site-port")
                                                            .cursor_pointer()
                                                            .flex()
                                                            .items_center()
                                                            .justify_between()
                                                            .px_2()
                                                            .py_1()
                                                            .rounded(px(4.0))
                                                            .bg(if active_field
                                                                == SiteEditField::Port
                                                            {
                                                                theme.button_active_bg()
                                                            } else {
                                                                theme.modal_card_bg()
                                                            })
                                                            .on_mouse_down(
                                                                MouseButton::Left,
                                                                cx.listener(|this, _, _, cx| {
                                                                    this.active_site_field =
                                                                        SiteEditField::Port;
                                                                    this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                    cx.notify();
                                                                }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.label_port()),
                                                            )
                                                            .child(render_field_value_with_cursor(
                                                                &if selected_site.port == 0 {
                                                                    String::new()
                                                                } else {
                                                                    format!("{}", selected_site.port)
                                                                },
                                                                active_field == SiteEditField::Port,
                                                                self.site_cursor_pos,
                                                                "21",
                                                                false,
                                                                is_dark,
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("edit-site-user")
                                                            .cursor_pointer()
                                                            .flex()
                                                            .items_center()
                                                            .justify_between()
                                                            .px_2()
                                                            .py_1()
                                                            .rounded(px(4.0))
                                                            .bg(if active_field
                                                                == SiteEditField::Username
                                                            {
                                                                theme.button_active_bg()
                                                            } else {
                                                                theme.modal_card_bg()
                                                            })
                                                            .on_mouse_down(
                                                                MouseButton::Left,
                                                                cx.listener(|this, _, _, cx| {
                                                                    this.active_site_field =
                                                                        SiteEditField::Username;
                                                                    this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                    cx.notify();
                                                                }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.label_user()),
                                                            )
                                                            .child(render_field_value_with_cursor(
                                                                &selected_site.username,
                                                                active_field == SiteEditField::Username,
                                                                self.site_cursor_pos,
                                                                "username",
                                                                false,
                                                                is_dark,
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("edit-site-pass")
                                                            .cursor_pointer()
                                                            .flex()
                                                            .items_center()
                                                            .justify_between()
                                                            .px_2()
                                                            .py_1()
                                                            .rounded(px(4.0))
                                                            .bg(if active_field
                                                                == SiteEditField::Password
                                                            {
                                                                theme.button_active_bg()
                                                            } else {
                                                                theme.modal_card_bg()
                                                            })
                                                            .on_mouse_down(
                                                                MouseButton::Left,
                                                                cx.listener(|this, _, _, cx| {
                                                                    this.active_site_field =
                                                                        SiteEditField::Password;
                                                                    this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                    cx.notify();
                                                                }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.label_password()),
                                                            )
                                                            .child(render_field_value_with_cursor(
                                                                if !self.site_password_input.is_empty() {
                                                                    &self.site_password_input
                                                                } else if let Some(ref p) = selected_site.password {
                                                                    p.as_str()
                                                                } else {
                                                                    ""
                                                                },
                                                                active_field == SiteEditField::Password,
                                                                self.site_cursor_pos,
                                                                "••••••••",
                                                                true,
                                                                is_dark,
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("edit-site-connections")
                                                            .flex()
                                                            .items_center()
                                                            .justify_between()
                                                            .px_2()
                                                            .py_1()
                                                            .rounded(px(4.0))
                                                            .bg(theme.modal_card_bg())
                                                            .child(
                                                                div()
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.label_max_connections()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap_1p5()
                                                                    .child(
                                                                        div()
                                                                            .id("site-conn-minus")
                                                                            .cursor_pointer()
                                                                            .w(px(20.0))
                                                                            .h(px(20.0))
                                                                            .rounded(px(4.0))
                                                                            .bg(theme.button_bg())
                                                                            .border_1()
                                                                            .border_color(theme.border())
                                                                            .flex()
                                                                            .items_center()
                                                                            .justify_center()
                                                                            .text_xs()
                                                                            .font_weight(gpui::FontWeight::BOLD)
                                                                            .text_color(theme.text_primary())
                                                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                                if let Some(s) = this.saved_sites.get_mut(this.selected_site_idx) {
                                                                                    if s.max_connections > 1 {
                                                                                        s.max_connections -= 1;
                                                                                        let _ = save_sites_to_disk(&this.saved_sites);
                                                                                        cx.notify();
                                                                                    }
                                                                                }
                                                                            }))
                                                                            .child("-"),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .w(px(28.0))
                                                                            .text_center()
                                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                                            .text_color(theme.text_primary())
                                                                            .child(format!("{}", selected_site.max_connections.clamp(1, 10))),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .id("site-conn-plus")
                                                                            .cursor_pointer()
                                                                            .w(px(20.0))
                                                                            .h(px(20.0))
                                                                            .rounded(px(4.0))
                                                                            .bg(theme.button_bg())
                                                                            .border_1()
                                                                            .border_color(theme.border())
                                                                            .flex()
                                                                            .items_center()
                                                                            .justify_center()
                                                                            .text_xs()
                                                                            .font_weight(gpui::FontWeight::BOLD)
                                                                            .text_color(theme.text_primary())
                                                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                                if let Some(s) = this.saved_sites.get_mut(this.selected_site_idx) {
                                                                                    if s.max_connections < 10 {
                                                                                        s.max_connections += 1;
                                                                                        let _ = save_sites_to_disk(&this.saved_sites);
                                                                                        cx.notify();
                                                                                    }
                                                                                }
                                                                            }))
                                                                            .child("+"),
                                                                    ),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .id("edit-site-env-color")
                                                            .flex()
                                                            .items_center()
                                                            .justify_between()
                                                            .px_2()
                                                            .py_1p5()
                                                            .rounded(px(4.0))
                                                            .bg(theme.modal_card_bg())
                                                            .child(
                                                                div()
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.label_environment()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap_1p5()
                                                                    .children(
                                                                        ENV_COLORS.iter().enumerate().map(|(ci, env_opt)| {
                                                                            let color_val = env_opt.color;
                                                                            let is_chosen = selected_site.env_color == color_val
                                                                                || (color_val == 0x2563eb && selected_site.env_color == 0x3b82f6)
                                                                                || (color_val == 0x10b981 && selected_site.env_color == 0x059669);

                                                                            div()
                                                                                .id(("env-color-swatch", ci))
                                                                                .cursor_pointer()
                                                                                .w(px(22.0))
                                                                                .h(px(22.0))
                                                                                .rounded(px(11.0))
                                                                                .bg(rgb(color_val))
                                                                                .flex()
                                                                                .items_center()
                                                                                .justify_center()
                                                                                .border_2()
                                                                                .border_color(if is_chosen {
                                                                                    if theme.is_dark() { rgb(0xffffff) } else { rgb(0x18181b) }
                                                                                } else {
                                                                                    rgba(0x00000020)
                                                                                })
                                                                                .opacity(if is_chosen { 1.0 } else { 0.75 })
                                                                                .hover(|s| s.opacity(1.0))
                                                                                .on_mouse_down(
                                                                                    MouseButton::Left,
                                                                                    cx.listener(move |this, _, _, cx| {
                                                                                        if let Some(s) = this.saved_sites.get_mut(this.selected_site_idx) {
                                                                                            s.env_color = color_val;
                                                                                            let _ = save_sites_to_disk(&this.saved_sites);
                                                                                            cx.notify();
                                                                                        }
                                                                                    }),
                                                                                )
                                                                                .when(is_chosen, |el| {
                                                                                    el.child(
                                                                                        div()
                                                                                            .w(px(7.0))
                                                                                            .h(px(7.0))
                                                                                            .rounded(px(3.5))
                                                                                            .bg(rgb(0xffffff)),
                                                                                    )
                                                                                })
                                                                        }),
                                                                    ),
                                                            ),
                                                    ),
                                            )
                                            .when_some(self.site_error_message.clone(), |el, err| {
                                                el.child(
                                                    div()
                                                        .px_3()
                                                        .py_2()
                                                        .rounded(px(5.0))
                                                        .bg(if theme.is_dark() { rgb(0x451a1a) } else { rgb(0xfef2f2) })
                                                        .border_1()
                                                        .border_color(if theme.is_dark() { rgb(0x7f1d1d) } else { rgb(0xfecaca) })
                                                        .text_xs()
                                                        .text_color(if theme.is_dark() { rgb(0xfca5a5) } else { rgb(0xdc2626) })
                                                        .child(format!("⚠️ {}", err)),
                                                )
                                            }),
                                    )
                                    .child(
                                        div()
                                            .flex_shrink_0()
                                            .pt_3()
                                            .border_t_1()
                                            .border_color(theme.border())
                                            .flex()
                                            .justify_end()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .id("btn-save-site")
                                                    .cursor_pointer()
                                                    .px_3()
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
                                                            this.sort_saved_sites();
                                                            let _ = save_sites_to_disk(
                                                                &this.saved_sites,
                                                            );
                                                            if let Some(site) = this
                                                                .saved_sites
                                                                .get(this.selected_site_idx)
                                                            {
                                                                if !this
                                                                    .site_password_input
                                                                    .is_empty()
                                                                {
                                                                    let _ =
                                                                        save_password_to_keychain(
                                                                            &site.host,
                                                                            &site.username,
                                                                            &this
                                                                                .site_password_input,
                                                                        );
                                                                }
                                                            }
                                                            this.active_site_field =
                                                                SiteEditField::None;
                                                            cx.notify();
                                                        }),
                                                    )
                                                    .child(t.btn_save()),
                                            )
                                            .child(
                                                div()
                                                    .id("btn-connect-now")
                                                    .cursor_pointer()
                                                    .px_4()
                                                    .py_1p5()
                                                    .rounded(px(5.0))
                                                    .bg(rgb(0x2563eb))
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(rgb(0xffffff))
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.connect_selected_site();
                                                            cx.notify();
                                                        }),
                                                    )
                                                    .child(t.btn_connect()),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}
