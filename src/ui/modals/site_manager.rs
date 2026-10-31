use gpui::{div, prelude::*, px, rgb, rgba, Context, IntoElement, MouseButton, MouseDownEvent};
use crate::app::{RusterView, SiteNoticeKind};
use crate::config::{
    import_from_ssh_config, save_password_to_keychain, save_sites_to_disk, SshImportResult,
};
use crate::icons::render_icon;
use crate::model::{ActiveModal, Protocol, SavedSiteProfile, SiteEditField, ENV_COLORS};
use crate::ui::helpers::{render_field_value_with_cursor, FileTooltip};

impl RusterView {
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

        let search_query = self.site_search_query.trim().to_lowercase();
        let filtered_indices: Vec<usize> = self
            .saved_sites
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                if search_query.is_empty() {
                    true
                } else {
                    s.name.to_lowercase().contains(&search_query)
                        || s.host.to_lowercase().contains(&search_query)
                        || s.username.to_lowercase().contains(&search_query)
                        || s.folder.to_lowercase().contains(&search_query)
                }
            })
            .map(|(i, _)| i)
            .collect();

        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(theme.modal_backdrop())
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                if this.site_proto_dropdown_open {
                    this.site_proto_dropdown_open = false;
                    cx.notify();
                }
                cx.stop_propagation();
            }))
            .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
            .child(
                div()
                    .w(px(780.0))
                    .h(px(520.0))
                    .rounded(px(10.0))
                    .bg(theme.modal_bg())
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    // --- MODAL HEADER ---
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_4()
                            .py_2p5()
                            .rounded_t(px(10.0))
                            .bg(theme.modal_header_bg())
                            .border_b_1()
                            .border_color(theme.border())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2p5()
                                    .child(
                                        div()
                                            .w(px(26.0))
                                            .h(px(26.0))
                                            .rounded(px(6.0))
                                            .bg(if is_dark { rgb(0x1e3a8a) } else { rgb(0xdbeafe) })
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(render_icon("icons/server.svg", 14.0, if is_dark { 0x60a5fa } else { 0x2563eb })),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(theme.text_primary())
                                                    .child(t.modal_site_manager_title()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.text_muted())
                                                    .child(t.modal_site_manager_subtitle()),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .id("close-site-modal")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .rounded(px(5.0))
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.sort_saved_sites();
                                            let _ = save_sites_to_disk(&this.saved_sites);
                                            this.active_modal = ActiveModal::None;
                                            this.active_site_field = SiteEditField::None;
                                            this.site_search_query.clear();
                                            this.site_proto_dropdown_open = false;
                                            this.site_notice = None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/close.svg", 14.0, theme.c_icon_default())),
                            ),
                    )
                    // --- BODY WORKSPACE (Sidebar + Right Form) ---
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .overflow_hidden()
                            // LEFT SIDEBAR
                            .child(
                                div()
                                    .w(px(260.0))
                                    .h_full()
                                    .bg(theme.modal_sidebar_bg())
                                    .border_r_1()
                                    .border_color(theme.border())
                                    .flex()
                                    .flex_col()
                                    .justify_between()
                                    // Search bar at top of sidebar
                                    .child(
                                        div()
                                            .p_2p5()
                                            .border_b_1()
                                            .border_color(theme.border())
                                            .child(
                                                div()
                                                    .id("site-search-box")
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .px_2p5()
                                                    .py_1p5()
                                                    .rounded(px(6.0))
                                                    .bg(theme.modal_card_bg())
                                                    .border_1()
                                                    .border_color(if active_field == SiteEditField::Search {
                                                        theme.button_active_border()
                                                    } else {
                                                        theme.border()
                                                    })
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.active_site_field = SiteEditField::Search;
                                                        this.site_cursor_pos = this.site_search_query.chars().count();
                                                        this.site_proto_dropdown_open = false;
                                                        cx.notify();
                                                    }))
                                                    .child(render_icon("icons/search.svg", 13.0, theme.c_icon_default()))
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .overflow_hidden()
                                                            .child(render_field_value_with_cursor(
                                                                &self.site_search_query,
                                                                active_field == SiteEditField::Search,
                                                                self.site_cursor_pos,
                                                                t.placeholder_search_sites(),
                                                                false,
                                                                is_dark,
                                                            )),
                                                    )
                                                    .when(!self.site_search_query.is_empty(), |el| {
                                                        el.child(
                                                            div()
                                                                .id("clear-site-search")
                                                                .cursor_pointer()
                                                                .w(px(16.0))
                                                                .h(px(16.0))
                                                                .rounded(px(3.0))
                                                                .flex()
                                                                .items_center()
                                                                .justify_center()
                                                                .hover(|s| s.bg(theme.button_hover_bg()))
                                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                    this.site_search_query.clear();
                                                                    this.site_cursor_pos = 0;
                                                                    cx.notify();
                                                                }))
                                                                .child(render_icon("icons/close.svg", 10.0, theme.c_icon_default())),
                                                        )
                                                    }),
                                            ),
                                    )
                                    // Scrollable list of sites
                                    .child(
                                        div()
                                            .id("site-list-scroll")
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .p_2()
                                            .overflow_y_scroll()
                                            .when(filtered_indices.is_empty(), |el| {
                                                el.child(
                                                    div()
                                                        .py_8()
                                                        .text_center()
                                                        .text_xs()
                                                        .text_color(theme.text_muted())
                                                        .child(t.no_matching_sites()),
                                                )
                                            })
                                            .children(filtered_indices.into_iter().map(|idx| {
                                                let is_sel = idx == self.selected_site_idx;
                                                let site = &self.saved_sites[idx];
                                                let proto = site.protocol;
                                                let env_col = site.env_color;
                                                let display_name = site.display_name().to_string();
                                                let path_subtext = if !site.folder.trim().is_empty() {
                                                    let folder = site.folder.trim();
                                                    if !site.host.trim().is_empty() {
                                                        if !site.username.trim().is_empty() {
                                                            format!("{}@{}:{}", site.username, site.host, folder)
                                                        } else {
                                                            format!("{}:{}", site.host, folder)
                                                        }
                                                    } else {
                                                        folder.to_string()
                                                    }
                                                } else if !site.host.trim().is_empty() {
                                                    if !site.username.trim().is_empty() {
                                                        format!("{}@{}", site.username, site.host)
                                                    } else {
                                                        site.host.clone()
                                                    }
                                                } else {
                                                    "/".to_string()
                                                };

                                                div()
                                                    .id(("site-item", idx))
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .justify_start()
                                                    .px_2p5()
                                                    .py_1p5()
                                                    .rounded(px(6.0))
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
                                                            this.active_site_field = SiteEditField::Name;
                                                            this.site_password_input = this
                                                                .saved_sites
                                                                .get(idx)
                                                                .and_then(|s| s.password.clone())
                                                                .unwrap_or_default();
                                                            this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                            this.site_error_message = None;
                                                            this.site_notice = None;
                                                            this.site_proto_dropdown_open = false;
                                                            if event.click_count >= 2 {
                                                                this.connect_selected_site(cx);
                                                            }
                                                            cx.notify();
                                                        }),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_2()
                                                            .overflow_hidden()
                                                            .child(
                                                                div()
                                                                    .w(px(8.0))
                                                                    .h(px(8.0))
                                                                    .rounded(px(4.0))
                                                                    .flex_shrink_0()
                                                                    .bg(rgb(env_col)),
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .overflow_hidden()
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .overflow_hidden()
                                                                            .text_ellipsis()
                                                                            .whitespace_nowrap()
                                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                                            .text_color(if is_sel {
                                                                                theme.row_text_selected()
                                                                            } else {
                                                                                theme.text_primary()
                                                                            })
                                                                            .child(display_name),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .overflow_hidden()
                                                                            .text_ellipsis()
                                                                            .whitespace_nowrap()
                                                                            .text_color(if is_sel {
                                                                                if is_dark { rgb(0x93c5fd) } else { rgb(0xdbeafe) }
                                                                            } else {
                                                                                theme.text_muted()
                                                                            })
                                                                            .child(path_subtext),
                                                                    ),
                                                            ),
                                                    )
                                            })),
                                    )
                                    // Bottom actions (+ Nový server, Import SSH, Del)
                                    .child(
                                        div()
                                            .p_2p5()
                                            .border_t_1()
                                            .border_color(theme.border())
                                            .bg(theme.modal_header_bg())
                                            .flex()
                                            .items_center()
                                            .justify_start()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .id("btn-new-site")
                                                    .cursor_pointer()
                                                    .w(px(28.0))
                                                    .h(px(28.0))
                                                    .rounded(px(6.0))
                                                    .bg(rgb(0x2563eb))
                                                    .hover(|s| s.bg(rgb(0x1d4ed8)))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .tooltip({
                                                        let tt = t.tooltip_new_server().to_string();
                                                        let is_dark = theme.is_dark();
                                                        move |_, cx| {
                                                            cx.new(|_| FileTooltip::simple(tt.clone(), is_dark)).into()
                                                        }
                                                    })
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.saved_sites.push(SavedSiteProfile {
                                                                name: String::new(),
                                                                folder: String::new(),
                                                                protocol: Protocol::Sftp,
                                                                host: String::new(),
                                                                port: 22,
                                                                username: String::new(),
                                                                password: None,
                                                                auth_type: String::new(),
                                                                env_color: 0x3b82f6,
                                                                max_connections: 2,
                                                            });
                                                            this.selected_site_idx = this.saved_sites.len() - 1;
                                                            this.draft_protocol = Protocol::Sftp;
                                                            this.active_site_field = SiteEditField::Name;
                                                            this.site_cursor_pos = 0;
                                                            this.site_password_input.clear();
                                                            this.site_error_message = None;
                                                            this.site_notice = None;
                                                            this.site_proto_dropdown_open = false;
                                                            this.sort_saved_sites();
                                                            let _ = save_sites_to_disk(&this.saved_sites);
                                                            cx.notify();
                                                        }),
                                                    )
                                                    .child(render_icon("icons/plus.svg", 14.0, 0xffffff)),
                                            )
                                            .child(
                                                div()
                                                    .id("btn-import-ssh")
                                                    .cursor_pointer()
                                                    .w(px(28.0))
                                                    .h(px(28.0))
                                                    .rounded(px(6.0))
                                                    .bg(theme.button_bg())
                                                    .border_1()
                                                    .border_color(theme.border())
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                                    .tooltip({
                                                        let tt = t.tooltip_import_ssh().to_string();
                                                        let is_dark = theme.is_dark();
                                                        move |_, cx| {
                                                            cx.new(|_| FileTooltip::simple(tt.clone(), is_dark)).into()
                                                        }
                                                    })
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            let t = this.i18n();
                                                            match import_from_ssh_config() {
                                                                SshImportResult::FileNotFound => {
                                                                    this.site_notice = Some((
                                                                        t.notice_ssh_config_not_found().to_string(),
                                                                        SiteNoticeKind::Error,
                                                                    ));
                                                                }
                                                                SshImportResult::NoHostsFound => {
                                                                    this.site_notice = Some((
                                                                        t.notice_ssh_config_no_hosts().to_string(),
                                                                        SiteNoticeKind::Error,
                                                                    ));
                                                                }
                                                                SshImportResult::Success(imported) => {
                                                                    let total = imported.len();
                                                                    let mut added = 0;
                                                                    for s in imported {
                                                                        if !this.saved_sites.iter().any(|existing| {
                                                                            existing.host == s.host && existing.username == s.username
                                                                        }) {
                                                                            this.saved_sites.push(s);
                                                                            added += 1;
                                                                        }
                                                                    }
                                                                    if added > 0 {
                                                                        this.sort_saved_sites();
                                                                        let _ = save_sites_to_disk(&this.saved_sites);
                                                                        this.site_notice = Some((
                                                                            t.notice_ssh_config_imported(added),
                                                                            SiteNoticeKind::Success,
                                                                        ));
                                                                    } else {
                                                                        this.site_notice = Some((
                                                                            t.notice_ssh_config_all_exist(total),
                                                                            SiteNoticeKind::Info,
                                                                        ));
                                                                    }
                                                                }
                                                            }
                                                            cx.notify();
                                                        }),
                                                    )
                                                    .child(render_icon("icons/folder-input.svg", 14.0, theme.c_icon_default())),
                                            )
                                            .child(
                                                div()
                                                    .id("btn-del-site")
                                                    .when(self.saved_sites.len() > 1, |el| el.cursor_pointer().hover(|s| s.bg(rgba(0xef444420))))
                                                    .when(self.saved_sites.len() <= 1, |el| el.opacity(0.4).cursor_default())
                                                    .w(px(28.0))
                                                    .h(px(28.0))
                                                    .rounded(px(6.0))
                                                    .bg(theme.button_bg())
                                                    .border_1()
                                                    .border_color(theme.border())
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .tooltip({
                                                        let tt = t.tooltip_delete_site().to_string();
                                                        let is_dark = theme.is_dark();
                                                        move |_, cx| {
                                                            cx.new(|_| FileTooltip::simple(tt.clone(), is_dark)).into()
                                                        }
                                                    })
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            if this.saved_sites.len() > 1 {
                                                                this.saved_sites.remove(this.selected_site_idx);
                                                                if this.selected_site_idx >= this.saved_sites.len() {
                                                                    this.selected_site_idx = this.saved_sites.len() - 1;
                                                                }
                                                                let _ = save_sites_to_disk(&this.saved_sites);
                                                                cx.notify();
                                                            }
                                                        }),
                                                    )
                                                    .child(render_icon("icons/trash.svg", 13.0, 0xef4444)),
                                            ),
                                    ),
                            )
                            // RIGHT FORM PANEL
                            .child(
                                div()
                                    .flex_1()
                                    .bg(theme.modal_bg())
                                    .flex()
                                    .flex_col()
                                    .justify_between()
                                    .overflow_hidden()
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        if this.site_proto_dropdown_open {
                                            this.site_proto_dropdown_open = false;
                                            cx.notify();
                                        }
                                    }))
                                    // Form Content Area
                                    .child(
                                        div()
                                            .id("site-form-scroll")
                                            .flex_1()
                                            .p_4()
                                            .overflow_y_scroll()
                                            .flex()
                                            .flex_col()
                                            .gap_4()
                                            // 1. Server Header (Color dot, Name, Subtitle, Env Swatches)
                                            .child(
                                                div()
                                                    .pb_3()
                                                    .border_b_1()
                                                    .border_color(theme.border())
                                                    .flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_3()
                                                            .child(
                                                                div()
                                                                    .w(px(20.0))
                                                                    .h(px(20.0))
                                                                    .rounded(px(10.0))
                                                                    .bg(rgb(selected_site.env_color))
                                                                    .shadow_sm(),
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .child(
                                                                        div()
                                                                            .id("edit-site-name")
                                                                            .cursor_pointer()
                                                                            .px_1p5()
                                                                            .py_0p5()
                                                                            .rounded(px(4.0))
                                                                            .bg(if active_field == SiteEditField::Name {
                                                                                theme.button_active_bg()
                                                                            } else {
                                                                                theme.modal_bg()
                                                                            })
                                                                            .border_1()
                                                                            .border_color(if active_field == SiteEditField::Name {
                                                                                theme.button_active_border()
                                                                            } else {
                                                                                rgba(0x00000000)
                                                                            })
                                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                                this.active_site_field = SiteEditField::Name;
                                                                                this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                                this.site_proto_dropdown_open = false;
                                                                                cx.notify();
                                                                            }))
                                                                            .child(
                                                                                div()
                                                                                    .font_weight(gpui::FontWeight::BOLD)
                                                                                    .child(render_field_value_with_cursor(
                                                                                        &selected_site.name,
                                                                                        active_field == SiteEditField::Name,
                                                                                        self.site_cursor_pos,
                                                                                        "Název připojení...",
                                                                                        false,
                                                                                        is_dark,
                                                                                    )),
                                                                            ),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .px_1p5()
                                                                            .text_xs()
                                                                            .text_color(theme.text_muted())
                                                                            .child(format!(
                                                                                "{} • {}@{}:{}",
                                                                                selected_site.protocol.label(),
                                                                                if selected_site.username.is_empty() { "user" } else { &selected_site.username },
                                                                                if selected_site.host.is_empty() { "host" } else { &selected_site.host },
                                                                                selected_site.port
                                                                            )),
                                                                    ),
                                                            ),
                                                    )
                                                    // Env color swatches on the right
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap_1p5()
                                                            .p_1()
                                                            .rounded(px(8.0))
                                                            .bg(theme.modal_sidebar_bg())
                                                            .border_1()
                                                            .border_color(theme.border())
                                                            .children(ENV_COLORS.iter().enumerate().map(|(ci, env_opt)| {
                                                                let color_val = env_opt.color;
                                                                let is_chosen = selected_site.env_color == color_val
                                                                    || (color_val == 0x2563eb && selected_site.env_color == 0x3b82f6)
                                                                    || (color_val == 0x10b981 && selected_site.env_color == 0x059669);

                                                                div()
                                                                    .id(("env-color-swatch", ci))
                                                                    .cursor_pointer()
                                                                    .w(px(20.0))
                                                                    .h(px(20.0))
                                                                    .rounded(px(10.0))
                                                                    .bg(rgb(color_val))
                                                                    .flex()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .border_2()
                                                                    .border_color(if is_chosen {
                                                                        if is_dark { rgb(0xffffff) } else { rgb(0x18181b) }
                                                                    } else {
                                                                        rgba(0x00000015)
                                                                    })
                                                                    .opacity(if is_chosen { 1.0 } else { 0.7 })
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
                                                                                .w(px(6.0))
                                                                                .h(px(6.0))
                                                                                .rounded(px(3.0))
                                                                                .bg(rgb(0xffffff)),
                                                                        )
                                                                    })
                                                            })),
                                                    ),
                                            )
                                            // 2. Row 1: Protocol Dropdown, Host Input, Port Input
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_3()
                                                    // Protocol dropdown
                                                    .child(
                                                        div()
                                                            .w(px(170.0))
                                                            .flex()
                                                            .flex_col()
                                                            .gap_1()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.label_protocol()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("site-proto-dropdown-btn")
                                                                    .cursor_pointer()
                                                                    .flex()
                                                                    .items_center()
                                                                    .justify_between()
                                                                    .px_2p5()
                                                                    .py_1p5()
                                                                    .rounded(px(6.0))
                                                                    .bg(theme.modal_card_bg())
                                                                    .border_1()
                                                                    .border_color(if self.site_proto_dropdown_open {
                                                                        theme.button_active_border()
                                                                    } else {
                                                                        theme.border()
                                                                    })
                                                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                        this.site_proto_dropdown_open = !this.site_proto_dropdown_open;
                                                                        cx.notify();
                                                                    }))
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                                            .text_color(theme.text_primary())
                                                                            .child(selected_site.protocol.label()),
                                                                    )
                                                                    .child(render_icon("icons/chevron-down.svg", 12.0, theme.c_icon_default())),
                                                            )
                                                            .when(self.site_proto_dropdown_open, |el| {
                                                                el.child(
                                                                    div()
                                                                        .id("site-proto-options-menu")
                                                                        .flex()
                                                                        .flex_col()
                                                                        .rounded(px(6.0))
                                                                        .bg(theme.modal_card_bg())
                                                                        .border_1()
                                                                        .border_color(theme.border_strong())
                                                                        .shadow_lg()
                                                                        .p_1()
                                                                        .gap_0p5()
                                                                        .children([
                                                                            Protocol::Sftp,
                                                                            Protocol::Scp,
                                                                            Protocol::FtpsExplicit,
                                                                            Protocol::FtpsImplicit,
                                                                            Protocol::Ftp,
                                                                        ].into_iter().enumerate().map(|(p_i, proto)| {
                                                                            let is_active = selected_site.protocol == proto;
                                                                            div()
                                                                                .id(("proto-opt", p_i))
                                                                                .cursor_pointer()
                                                                                .flex()
                                                                                .items_center()
                                                                                .justify_between()
                                                                                .px_2()
                                                                                .py_1()
                                                                                .rounded(px(4.0))
                                                                                .bg(if is_active {
                                                                                    theme.button_active_bg()
                                                                                } else {
                                                                                    theme.modal_card_bg()
                                                                                })
                                                                                .hover(|s| s.bg(theme.button_hover_bg()))
                                                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                                                    this.draft_protocol = proto;
                                                                                    if let Some(s) = this.saved_sites.get_mut(this.selected_site_idx) {
                                                                                        s.protocol = proto;
                                                                                        s.port = proto.default_port();
                                                                                    }
                                                                                    this.site_proto_dropdown_open = false;
                                                                                    cx.notify();
                                                                                }))
                                                                                .child(
                                                                                    div()
                                                                                        .text_xs()
                                                                                        .font_weight(gpui::FontWeight::MEDIUM)
                                                                                        .text_color(if is_active {
                                                                                            theme.text_accent()
                                                                                        } else {
                                                                                            theme.text_primary()
                                                                                        })
                                                                                        .child(proto.label()),
                                                                                )
                                                                                .when(is_active, |opt| {
                                                                                    opt.child(render_icon("icons/check.svg", 12.0, theme.c_icon_active()))
                                                                                })
                                                                        })),
                                                                )
                                                            }),
                                                    )
                                                    // Host input
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
                                                                    .child(t.label_host()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("edit-site-host")
                                                                    .cursor_pointer()
                                                                    .px_2p5()
                                                                    .py_1p5()
                                                                    .rounded(px(6.0))
                                                                    .bg(theme.modal_card_bg())
                                                                    .border_1()
                                                                    .border_color(if active_field == SiteEditField::Host {
                                                                        theme.button_active_border()
                                                                    } else {
                                                                        theme.border()
                                                                    })
                                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                        this.active_site_field = SiteEditField::Host;
                                                                        this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                        this.site_proto_dropdown_open = false;
                                                                        cx.notify();
                                                                    }))
                                                                    .child(render_field_value_with_cursor(
                                                                        &selected_site.host,
                                                                        active_field == SiteEditField::Host,
                                                                        self.site_cursor_pos,
                                                                        "prod.cloud.cz",
                                                                        false,
                                                                        is_dark,
                                                                    )),
                                                            ),
                                                    )
                                                    // Port input
                                                    .child(
                                                        div()
                                                            .w(px(74.0))
                                                            .flex()
                                                            .flex_col()
                                                            .gap_1()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.label_port()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("edit-site-port")
                                                                    .cursor_pointer()
                                                                    .px_2p5()
                                                                    .py_1p5()
                                                                    .rounded(px(6.0))
                                                                    .bg(theme.modal_card_bg())
                                                                    .border_1()
                                                                    .border_color(if active_field == SiteEditField::Port {
                                                                        theme.button_active_border()
                                                                    } else {
                                                                        theme.border()
                                                                    })
                                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                        this.active_site_field = SiteEditField::Port;
                                                                        this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                        this.site_proto_dropdown_open = false;
                                                                        cx.notify();
                                                                    }))
                                                                    .child(render_field_value_with_cursor(
                                                                        &if selected_site.port == 0 {
                                                                            String::new()
                                                                        } else {
                                                                            selected_site.port.to_string()
                                                                        },
                                                                        active_field == SiteEditField::Port,
                                                                        self.site_cursor_pos,
                                                                        "22",
                                                                        false,
                                                                        is_dark,
                                                                    )),
                                                            ),
                                                    ),
                                            )
                                            // 3. Row 2: Username & Password
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_3()
                                                    // Username
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
                                                                    .child(t.label_user()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("edit-site-user")
                                                                    .cursor_pointer()
                                                                    .px_2p5()
                                                                    .py_1p5()
                                                                    .rounded(px(6.0))
                                                                    .bg(theme.modal_card_bg())
                                                                    .border_1()
                                                                    .border_color(if active_field == SiteEditField::Username {
                                                                        theme.button_active_border()
                                                                    } else {
                                                                        theme.border()
                                                                    })
                                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                        this.active_site_field = SiteEditField::Username;
                                                                        this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                        this.site_proto_dropdown_open = false;
                                                                        cx.notify();
                                                                    }))
                                                                    .child(render_field_value_with_cursor(
                                                                        &selected_site.username,
                                                                        active_field == SiteEditField::Username,
                                                                        self.site_cursor_pos,
                                                                        "root",
                                                                        false,
                                                                        is_dark,
                                                                    )),
                                                            ),
                                                    )
                                                    // Password
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .flex()
                                                            .flex_col()
                                                            .gap_1()
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .items_center()
                                                                    .justify_between()
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                                            .text_color(theme.text_secondary())
                                                                            .child(t.label_password()),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .id("toggle-pass-visibility")
                                                                            .cursor_pointer()
                                                                            .text_xs()
                                                                            .text_color(theme.text_accent())
                                                                            .hover(|s| s.underline())
                                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                                this.site_password_revealed = !this.site_password_revealed;
                                                                                cx.notify();
                                                                            }))
                                                                            .child(if self.site_password_revealed {
                                                                                t.btn_toggle_hide_pass()
                                                                            } else {
                                                                                t.btn_toggle_show_pass()
                                                                            }),
                                                                    ),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("edit-site-pass")
                                                                    .cursor_pointer()
                                                                    .px_2p5()
                                                                    .py_1p5()
                                                                    .rounded(px(6.0))
                                                                    .bg(theme.modal_card_bg())
                                                                    .border_1()
                                                                    .border_color(if active_field == SiteEditField::Password {
                                                                        theme.button_active_border()
                                                                    } else {
                                                                        theme.border()
                                                                    })
                                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                        this.active_site_field = SiteEditField::Password;
                                                                        this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                        this.site_proto_dropdown_open = false;
                                                                        cx.notify();
                                                                    }))
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
                                                                        !self.site_password_revealed,
                                                                        is_dark,
                                                                    )),
                                                            ),
                                                    ),
                                            )
                                            // 4. Row 3: Initial Remote Directory & Max Concurrent Transfers
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_3()
                                                    // Initial remote directory
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
                                                                    .child(t.label_initial_remote_path()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("edit-site-folder")
                                                                    .cursor_pointer()
                                                                    .px_2p5()
                                                                    .py_1p5()
                                                                    .rounded(px(6.0))
                                                                    .bg(theme.modal_card_bg())
                                                                    .border_1()
                                                                    .border_color(if active_field == SiteEditField::RemotePath {
                                                                        theme.button_active_border()
                                                                    } else {
                                                                        theme.border()
                                                                    })
                                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                        this.active_site_field = SiteEditField::RemotePath;
                                                                        this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                        this.site_proto_dropdown_open = false;
                                                                        cx.notify();
                                                                    }))
                                                                    .child(render_field_value_with_cursor(
                                                                        &selected_site.folder,
                                                                        active_field == SiteEditField::RemotePath,
                                                                        self.site_cursor_pos,
                                                                        "/var/www/html nebo ~",
                                                                        false,
                                                                        is_dark,
                                                                    )),
                                                            ),
                                                    )
                                                    // Max concurrent connections (Number input with stepper buttons)
                                                    .child(
                                                        div()
                                                            .w(px(130.0))
                                                            .flex()
                                                            .flex_col()
                                                            .gap_1()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                                    .text_color(theme.text_secondary())
                                                                    .child(t.label_max_connections_short()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("edit-site-connections")
                                                                    .flex()
                                                                    .items_center()
                                                                    .justify_between()
                                                                    .px_2()
                                                                    .py_1()
                                                                    .rounded(px(6.0))
                                                                    .bg(theme.modal_card_bg())
                                                                    .border_1()
                                                                    .border_color(if active_field == SiteEditField::MaxConnections {
                                                                        theme.button_active_border()
                                                                    } else {
                                                                        theme.border()
                                                                    })
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
                                                                            .cursor_pointer()
                                                                            .px_1()
                                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                                                this.active_site_field = SiteEditField::MaxConnections;
                                                                                this.site_cursor_pos = this.current_site_field_val().chars().count();
                                                                                cx.notify();
                                                                            }))
                                                                            .child(render_field_value_with_cursor(
                                                                                &selected_site.max_connections.to_string(),
                                                                                active_field == SiteEditField::MaxConnections,
                                                                                self.site_cursor_pos,
                                                                                "2",
                                                                                false,
                                                                                is_dark,
                                                                            )),
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
                                                    ),
                                            )
                                            // Status / Notice / Error Alerts
                                            .when_some(self.site_notice.clone(), |el, (msg, kind)| {
                                                let (bg, border, text_color, icon_str) = match kind {
                                                    SiteNoticeKind::Success => (
                                                        if theme.is_dark() { rgb(0x064e3b) } else { rgb(0xecfdf5) },
                                                        if theme.is_dark() { rgb(0x047857) } else { rgb(0xa7f3d0) },
                                                        if theme.is_dark() { rgb(0x6ee7b7) } else { rgb(0x047857) },
                                                        "✅ ",
                                                    ),
                                                    SiteNoticeKind::Info => (
                                                        if theme.is_dark() { rgb(0x1e293b) } else { rgb(0xf0fdf4) },
                                                        if theme.is_dark() { rgb(0x334155) } else { rgb(0xcfdfe8) },
                                                        if theme.is_dark() { rgb(0x93c5fd) } else { rgb(0x0369a1) },
                                                        "ℹ️ ",
                                                    ),
                                                    SiteNoticeKind::Error => (
                                                        if theme.is_dark() { rgb(0x451a1a) } else { rgb(0xfef2f2) },
                                                        if theme.is_dark() { rgb(0x7f1d1d) } else { rgb(0xfecaca) },
                                                        if theme.is_dark() { rgb(0xfca5a5) } else { rgb(0xdc2626) },
                                                        "⚠️ ",
                                                    ),
                                                };
                                                el.child(
                                                    div()
                                                        .px_3()
                                                        .py_2()
                                                        .rounded(px(6.0))
                                                        .bg(bg)
                                                        .border_1()
                                                        .border_color(border)
                                                        .text_xs()
                                                        .text_color(text_color)
                                                        .child(format!("{}{}", icon_str, msg)),
                                                )
                                            })
                                            .when(self.site_is_connecting, |el| {
                                                let host = selected_site.host.as_str();
                                                let port = selected_site.port;
                                                el.child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_2()
                                                        .px_3()
                                                        .py_2()
                                                        .rounded(px(6.0))
                                                        .bg(if theme.is_dark() { rgb(0x172554) } else { rgb(0xeff6ff) })
                                                        .border_1()
                                                        .border_color(if theme.is_dark() { rgb(0x1e40af) } else { rgb(0xbfdbfe) })
                                                        .text_xs()
                                                        .text_color(if theme.is_dark() { rgb(0x93c5fd) } else { rgb(0x1d4ed8) })
                                                        .child(render_icon("icons/sync.svg", 14.0, theme.c_icon_active()))
                                                        .child(t.site_connecting_status(host, port)),
                                                )
                                            })
                                            .when_some(self.site_error_message.clone(), |el, err| {
                                                el.child(
                                                    div()
                                                        .px_3()
                                                        .py_2()
                                                        .rounded(px(6.0))
                                                        .bg(if theme.is_dark() { rgb(0x451a1a) } else { rgb(0xfef2f2) })
                                                        .border_1()
                                                        .border_color(if theme.is_dark() { rgb(0x7f1d1d) } else { rgb(0xfecaca) })
                                                        .text_xs()
                                                        .text_color(if theme.is_dark() { rgb(0xfca5a5) } else { rgb(0xdc2626) })
                                                        .child(format!("⚠️ {}", err)),
                                                )
                                            }),
                                    )
                                    // BOTTOM ACTION FOOTER (Uložit změny s disketou + Připojit se)
                                    .child(
                                        div()
                                            .h(px(48.0))
                                            .px_4()
                                            .border_t_1()
                                            .border_color(theme.border())
                                            .bg(theme.modal_header_bg())
                                            .flex()
                                            .items_center()
                                            .justify_end()
                                            .gap_2()
                                            // Save button with Floppy Disk icon
                                            .child(
                                                div()
                                                    .id("btn-save-site")
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .px_3p5()
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
                                                            this.sort_saved_sites();
                                                            let _ = save_sites_to_disk(&this.saved_sites);
                                                            if let Some(site) = this.saved_sites.get(this.selected_site_idx) {
                                                                if !this.site_password_input.is_empty() {
                                                                    let _ = save_password_to_keychain(
                                                                        &site.host,
                                                                        &site.username,
                                                                        &this.site_password_input,
                                                                    );
                                                                }
                                                            }
                                                            this.active_site_field = SiteEditField::None;
                                                            this.site_proto_dropdown_open = false;
                                                            cx.notify();
                                                        }),
                                                    )
                                                    .child(render_icon("icons/save.svg", 13.0, theme.c_icon_default()))
                                                    .child(t.btn_save()),
                                            )
                                            // Connect button
                                            .child(
                                                div()
                                                    .id("btn-connect-now")
                                                    .when(!self.site_is_connecting, |el| el.cursor_pointer().hover(|s| s.bg(rgb(0x1d4ed8))))
                                                    .when(self.site_is_connecting, |el| el.cursor_default().opacity(0.8))
                                                    .px_4()
                                                    .py_1p5()
                                                    .rounded(px(6.0))
                                                    .bg(rgb(0x2563eb))
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(rgb(0xffffff))
                                                    .shadow_sm()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1p5()
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            if !this.site_is_connecting {
                                                                this.connect_selected_site(cx);
                                                            }
                                                        }),
                                                    )
                                                    .when(self.site_is_connecting, |el| {
                                                        el.child(render_icon("icons/sync.svg", 12.0, 0xffffff))
                                                    })
                                                    .child(if self.site_is_connecting {
                                                        t.btn_connecting()
                                                    } else {
                                                        t.btn_connect()
                                                    })
                                                    .when(!self.site_is_connecting, |el| {
                                                        el.child(
                                                            div()
                                                                .px_1()
                                                                .rounded(px(3.0))
                                                                .bg(rgba(0xffffff30))
                                                                .text_xs()
                                                                .text_color(rgb(0xffffff))
                                                                .child("↵"),
                                                        )
                                                    }),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}
