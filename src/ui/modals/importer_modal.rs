use gpui::{
    div, prelude::*, px, rgb, rgba, Context, FontWeight, IntoElement, MouseButton,
};
use std::fs;
use std::path::PathBuf;

use crate::app::RusterView;
use crate::config::{save_favorite_paths, save_password_to_keychain, save_sites_to_disk};
use crate::icons::render_icon;
use crate::importer::{
    create_ruster_backup, parse_ruster_backup, parse_winscp_content, site_backup_to_profile,
    ImportSourceType, ImportSummary,
};
use crate::model::ActiveModal;

impl RusterView {
    pub fn open_import_modal(&mut self) {
        self.active_modal = ActiveModal::ImportConfig;
        self.import_summary = None;
        if self.import_source == ImportSourceType::SshConfig && self.import_file_path.is_empty() {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
            self.import_file_path = format!("{}/.ssh/config", home);
        }
    }

    pub fn open_export_modal(&mut self) {
        self.active_modal = ActiveModal::ExportConfig;
        self.export_status = None;
    }

    pub fn render_import_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();

        let source = self.import_source;
        let file_path = self.import_file_path.clone();

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
                    .rounded(px(8.0))
                    .bg(theme.modal_bg())
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_lg()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_3p5()
                    // Modal Header
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
                                    .child(render_icon("icons/folder-input.svg", 16.0, theme.c_icon_active()))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme.text_primary())
                                            .child(t.modal_import_title()),
                                    ),
                            )
                            .child(
                                div()
                                    .id("btn-close-import-modal")
                                    .cursor_pointer()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .rounded(px(4.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.active_modal = ActiveModal::None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/x.svg", 14.0, theme.c_icon_default())),
                            ),
                    )
                    // Source Selector Tabs
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text_secondary())
                                    .child(t.import_source_label()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(self.render_source_pill(
                                        ImportSourceType::WinScp,
                                        "WinSCP (.ini / .reg)",
                                        cx,
                                    ))
                                    .child(self.render_source_pill(
                                        ImportSourceType::RusterBackup,
                                        "Ruster (JSON)",
                                        cx,
                                    ))
                                    .child(self.render_source_pill(
                                        ImportSourceType::SshConfig,
                                        "OpenSSH (~/.ssh/config)",
                                        cx,
                                    )),
                            ),
                    )
                    // File path input & Browse button
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .flex_1()
                                            .px_2p5()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.input_bg())
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .text_color(if file_path.is_empty() {
                                                theme.text_muted()
                                            } else {
                                                theme.text_primary()
                                            })
                                            .child(if file_path.is_empty() {
                                                match source {
                                                    ImportSourceType::WinScp => "Vyberte soubor WinSCP.ini nebo .reg...",
                                                    ImportSourceType::RusterBackup => "Vyberte soubor ruster-backup.json...",
                                                    ImportSourceType::SshConfig => "~/.ssh/config",
                                                }
                                                .to_string()
                                            } else {
                                                file_path.clone()
                                            }),
                                    )
                                    .child(
                                        div()
                                            .id("btn-browse-import-file")
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
                                                cx.listener(|_, _, _, cx| {
                                                    let rx = cx.prompt_for_paths(gpui::PathPromptOptions {
                                                        files: true,
                                                        directories: false,
                                                        multiple: false,
                                                        prompt: None,
                                                    });
                                                    cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
                                                        let mut cx = cx.clone();
                                                        async move {
                                                            if let Ok(Ok(Some(paths))) = rx.await {
                                                                if let Some(path) = paths.into_iter().next() {
                                                                    let _ = this.update(&mut cx, |view, cx| {
                                                                        view.import_file_path = path.to_string_lossy().into_owned();
                                                                        view.import_summary = None;
                                                                        cx.notify();
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    })
                                                    .detach();
                                                }),
                                            )
                                            .child(t.btn_browse()),
                                    ),
                            ),
                    )
                    // Checkbox options
                    .child(
                        div()
                            .p_2p5()
                            .rounded(px(6.0))
                            .bg(theme.popover_bg())
                            .border_1()
                            .border_color(theme.border())
                            .flex()
                            .flex_col()
                            .gap_2()
                            .children(self.render_import_checkboxes(source, cx)),
                    )
                    // Summary or Notice
                    .when_some(self.import_summary.clone(), |el, summary| {
                        el.child(self.render_import_summary_banner(&summary))
                    })
                    // Bottom Buttons
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .pt_1()
                            .child(
                                div()
                                    .id("btn-cancel-import")
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
                                            this.active_modal = ActiveModal::None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(t.btn_cancel()),
                            )
                            .child(
                                div()
                                    .id("btn-perform-import")
                                    .cursor_pointer()
                                    .px_4()
                                    .py_1p5()
                                    .rounded(px(5.0))
                                    .bg(rgb(0x2563eb))
                                    .hover(|s| s.bg(rgb(0x1d4ed8)))
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xffffff))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.execute_import(cx);
                                        }),
                                    )
                                    .child(t.btn_import_action()),
                            ),
                    ),
            )
    }

    fn render_source_pill(
        &self,
        source: ImportSourceType,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = self.import_source == source;
        let theme = self.theme();
        let idx = match source {
            ImportSourceType::WinScp => 0usize,
            ImportSourceType::RusterBackup => 1usize,
            ImportSourceType::SshConfig => 2usize,
        };

        div()
            .id(("import-source-pill", idx))
            .cursor_pointer()
            .px_3()
            .py_1p5()
            .rounded(px(5.0))
            .border_1()
            .border_color(if is_selected {
                rgb(0x2563eb)
            } else {
                theme.border()
            })
            .bg(if is_selected {
                if theme.is_dark() {
                    rgba(0x2563eb30)
                } else {
                    rgba(0x2563eb18)
                }
            } else {
                theme.button_bg()
            })
            .text_xs()
            .font_weight(if is_selected {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(if is_selected {
                theme.text_accent()
            } else {
                theme.text_primary()
            })
            .hover(|s| s.bg(theme.button_hover_bg()))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.import_source = source;
                    this.import_summary = None;
                    if source == ImportSourceType::SshConfig {
                        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
                        this.import_file_path = format!("{}/.ssh/config", home);
                    } else if this.import_file_path.ends_with(".ssh/config") {
                        this.import_file_path.clear();
                    }
                    cx.notify();
                }),
            )
            .child(label)
    }

    fn render_import_checkboxes(
        &self,
        source: ImportSourceType,
        cx: &mut Context<Self>,
    ) -> Vec<impl IntoElement> {
        let t = self.i18n();
        let mut items = Vec::new();

        match source {
            ImportSourceType::WinScp => {
                let save_pwd = self.import_options.save_passwords_to_keychain;
                items.push(
                    div()
                        .id("cb-winscp-pwd")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.import_options.save_passwords_to_keychain =
                                    !this.import_options.save_passwords_to_keychain;
                                cx.notify();
                            }),
                        )
                        .child(self.render_checkbox_mark(save_pwd))
                        .child(
                            div()
                                .text_xs()
                                .text_color(self.theme().text_primary())
                                .child(t.import_opt_passwords()),
                        ),
                );
            }
            ImportSourceType::RusterBackup => {
                let opt_sites = self.import_options.import_sites;
                items.push(
                    div()
                        .id("cb-ruster-sites")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.import_options.import_sites = !this.import_options.import_sites;
                                cx.notify();
                            }),
                        )
                        .child(self.render_checkbox_mark(opt_sites))
                        .child(
                            div()
                                .text_xs()
                                .text_color(self.theme().text_primary())
                                .child(t.import_opt_sites()),
                        ),
                );

                let opt_pwd = self.import_options.save_passwords_to_keychain;
                items.push(
                    div()
                        .id("cb-ruster-pwd")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.import_options.save_passwords_to_keychain =
                                    !this.import_options.save_passwords_to_keychain;
                                cx.notify();
                            }),
                        )
                        .child(self.render_checkbox_mark(opt_pwd))
                        .child(
                            div()
                                .text_xs()
                                .text_color(self.theme().text_primary())
                                .child(t.import_opt_passwords()),
                        ),
                );

                let opt_settings = self.import_options.import_settings;
                items.push(
                    div()
                        .id("cb-ruster-settings")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.import_options.import_settings =
                                    !this.import_options.import_settings;
                                cx.notify();
                            }),
                        )
                        .child(self.render_checkbox_mark(opt_settings))
                        .child(
                            div()
                                .text_xs()
                                .text_color(self.theme().text_primary())
                                .child(t.import_opt_settings()),
                        ),
                );

                let opt_favs = self.import_options.import_favorites;
                items.push(
                    div()
                        .id("cb-ruster-favs")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.import_options.import_favorites =
                                    !this.import_options.import_favorites;
                                cx.notify();
                            }),
                        )
                        .child(self.render_checkbox_mark(opt_favs))
                        .child(
                            div()
                                .text_xs()
                                .text_color(self.theme().text_primary())
                                .child(t.import_opt_favorites()),
                        ),
                );
            }
            ImportSourceType::SshConfig => {
                let opt_sites = self.import_options.import_sites;
                items.push(
                    div()
                        .id("cb-ssh-sites")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.import_options.import_sites = !this.import_options.import_sites;
                                cx.notify();
                            }),
                        )
                        .child(self.render_checkbox_mark(opt_sites))
                        .child(
                            div()
                                .text_xs()
                                .text_color(self.theme().text_primary())
                                .child(t.import_opt_sites()),
                        ),
                );
            }
        }

        items
    }

    fn render_checkbox_mark(&self, checked: bool) -> impl IntoElement {
        div()
            .w(px(16.0))
            .h(px(16.0))
            .rounded(px(4.0))
            .border_1()
            .border_color(if checked {
                rgb(0x2563eb)
            } else {
                self.theme().border()
            })
            .bg(if checked {
                rgb(0x2563eb)
            } else {
                self.theme().input_bg()
            })
            .flex()
            .items_center()
            .justify_center()
            .when(checked, |el| {
                el.child(render_icon("icons/check.svg", 12.0, 0xffffff))
            })
    }

    fn render_import_summary_banner(&self, summary: &ImportSummary) -> impl IntoElement {
        let is_error = summary.error.is_some();
        let (bg, border, text_color) = if is_error {
            (rgba(0xef444415), rgba(0xef444440), rgb(0xef4444))
        } else {
            (rgba(0x10b98115), rgba(0x10b98140), rgb(0x10b981))
        };

        let msg = if let Some(ref err) = summary.error {
            err.clone()
        } else {
            format!(
                "Úspěšně importováno: {} serverů (uloženo {} hesel), {} přeskočeno.{}",
                summary.servers_imported,
                summary.passwords_saved,
                summary.servers_skipped,
                if summary.settings_imported {
                    " Nastavení obnoveno."
                } else {
                    ""
                }
            )
        };

        div()
            .p_2p5()
            .rounded(px(6.0))
            .bg(bg)
            .border_1()
            .border_color(border)
            .flex()
            .items_center()
            .gap_2()
            .child(render_icon(
                if is_error {
                    "icons/alert-circle.svg"
                } else {
                    "icons/check.svg"
                },
                14.0,
                if is_error { 0xef4444 } else { 0x10b981 },
            ))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(msg),
            )
    }

    pub fn execute_import(&mut self, cx: &mut Context<Self>) {
        let path_str = self.import_file_path.trim().to_string();
        if path_str.is_empty() {
            self.import_summary = Some(ImportSummary {
                error: Some("Zadejte prosím cestu k souboru nebo jej vyberte.".to_string()),
                ..Default::default()
            });
            cx.notify();
            return;
        }

        // Expand ~ to HOME directory
        let expanded = if let Some(stripped) = path_str.strip_prefix("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
            PathBuf::from(home).join(stripped)
        } else {
            PathBuf::from(&path_str)
        };

        if !expanded.exists() {
            self.import_summary = Some(ImportSummary {
                error: Some(format!("Soubor neexistuje: {}", path_str)),
                ..Default::default()
            });
            cx.notify();
            return;
        }

        let content = match fs::read_to_string(&expanded) {
            Ok(c) => c,
            Err(e) => {
                self.import_summary = Some(ImportSummary {
                    error: Some(format!("Nelze přečíst soubor: {}", e)),
                    ..Default::default()
                });
                cx.notify();
                return;
            }
        };

        match self.import_source {
            ImportSourceType::WinScp => {
                let sites = parse_winscp_content(&content);
                if sites.is_empty() {
                    self.import_summary = Some(ImportSummary {
                        error: Some("V souboru nebyly nalezeny žádné relace WinSCP.".to_string()),
                        ..Default::default()
                    });
                    cx.notify();
                    return;
                }

                let mut imported = 0;
                let mut skipped = 0;
                let mut passwords_saved = 0;

                for site in sites {
                    let exists = self.saved_sites.iter().any(|existing| {
                        existing.host == site.host && existing.username == site.username && existing.port == site.port
                    });

                    if exists {
                        skipped += 1;
                        continue;
                    }

                    if self.import_options.save_passwords_to_keychain {
                        if let Some(ref pwd) = site.password {
                            if !pwd.is_empty() && save_password_to_keychain(&site.host, &site.username, pwd) {
                                passwords_saved += 1;
                            }
                        }
                    }

                    // Keep password in memory profile
                    self.saved_sites.push(site);
                    imported += 1;
                }

                if imported > 0 {
                    self.sort_saved_sites();
                    let _ = save_sites_to_disk(&self.saved_sites);
                }

                self.import_summary = Some(ImportSummary {
                    servers_imported: imported,
                    servers_skipped: skipped,
                    passwords_saved,
                    settings_imported: false,
                    favorites_imported: 0,
                    error: None,
                });
            }
            ImportSourceType::RusterBackup => {
                let backup = match parse_ruster_backup(&content) {
                    Ok(b) => b,
                    Err(e) => {
                        self.import_summary = Some(ImportSummary {
                            error: Some(format!("Neplatná záloha Rusteru: {}", e)),
                            ..Default::default()
                        });
                        cx.notify();
                        return;
                    }
                };

                let mut imported = 0;
                let mut skipped = 0;
                let mut passwords_saved = 0;

                if self.import_options.import_sites {
                    if let Some(sites) = backup.sites {
                        for site_b in sites {
                            let site = site_backup_to_profile(&site_b);
                            let exists = self.saved_sites.iter().any(|existing| {
                                existing.host == site.host && existing.username == site.username && existing.port == site.port
                            });

                            if exists {
                                skipped += 1;
                                continue;
                            }

                            if self.import_options.save_passwords_to_keychain {
                                if let Some(ref pwd) = site.password {
                                    if !pwd.is_empty() && save_password_to_keychain(&site.host, &site.username, pwd) {
                                        passwords_saved += 1;
                                    }
                                }
                            }

                            self.saved_sites.push(site);
                            imported += 1;
                        }
                        if imported > 0 {
                            self.sort_saved_sites();
                            let _ = save_sites_to_disk(&self.saved_sites);
                        }
                    }
                }

                let mut settings_imported = false;
                if self.import_options.import_settings {
                    if let Some(s) = backup.settings {
                        self.show_local_pane = s.show_local_pane;
                        self.show_hidden = s.show_hidden;
                        self.show_function_bar = s.show_function_bar;
                        self.show_permissions = s.show_permissions;
                        self.date_format = crate::model::DateFormat::from_str(&s.date_format);
                        self.max_concurrent_transfers = s.max_concurrent_transfers.clamp(1, 10);
                        self.editable_extensions = s.editable_extensions;
                        self.theme = crate::theme::ThemeMode::from_str(&s.theme);
                        self.locale = crate::i18n::Locale::from_code(&s.locale);
                        self.persist_app_state();
                        crate::ui::menus::install_app_menus(cx, self.locale);
                        settings_imported = true;
                    }
                }

                let mut favs_count = 0;
                if self.import_options.import_favorites {
                    if let Some(favs) = backup.favorites {
                        favs_count = favs.len();
                        self.favorite_paths = favs;
                        let _ = save_favorite_paths(&self.favorite_paths);
                    }
                }

                self.import_summary = Some(ImportSummary {
                    servers_imported: imported,
                    servers_skipped: skipped,
                    passwords_saved,
                    settings_imported,
                    favorites_imported: favs_count,
                    error: None,
                });
            }
            ImportSourceType::SshConfig => {
                let sites = crate::config::parse_ssh_config_str(&content);
                if sites.is_empty() {
                    self.import_summary = Some(ImportSummary {
                        error: Some("V souboru nebyly nalezeny žádné SSH servery.".to_string()),
                        ..Default::default()
                    });
                    cx.notify();
                    return;
                }

                let mut imported = 0;
                let mut skipped = 0;

                for site in sites {
                    let exists = self.saved_sites.iter().any(|existing| {
                        existing.host == site.host && existing.username == site.username && existing.port == site.port
                    });

                    if exists {
                        skipped += 1;
                        continue;
                    }

                    self.saved_sites.push(site);
                    imported += 1;
                }

                if imported > 0 {
                    self.sort_saved_sites();
                    let _ = save_sites_to_disk(&self.saved_sites);
                }

                self.import_summary = Some(ImportSummary {
                    servers_imported: imported,
                    servers_skipped: skipped,
                    passwords_saved: 0,
                    settings_imported: false,
                    favorites_imported: 0,
                    error: None,
                });
            }
        }

        cx.notify();
    }

    // -----------------------------------------------------------------------
    // Export Modal
    // -----------------------------------------------------------------------

    pub fn render_export_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();

        let opt_sites = self.export_options.export_sites;
        let opt_pwd = self.export_options.export_passwords;
        let opt_settings = self.export_options.export_settings;
        let opt_favs = self.export_options.export_favorites;

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
                    .gap_3p5()
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
                                    .child(render_icon("icons/folder-output.svg", 16.0, theme.c_icon_active()))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme.text_primary())
                                            .child(t.modal_export_title()),
                                    ),
                            )
                            .child(
                                div()
                                    .id("btn-close-export-modal")
                                    .cursor_pointer()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .rounded(px(4.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.active_modal = ActiveModal::None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/x.svg", 14.0, theme.c_icon_default())),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_secondary())
                            .child(t.export_description()),
                    )
                    // Checkboxes
                    .child(
                        div()
                            .p_3()
                            .rounded(px(6.0))
                            .bg(theme.popover_bg())
                            .border_1()
                            .border_color(theme.border())
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            // Option 1: Sites
                            .child(
                                div()
                                    .id("cb-export-sites")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.export_options.export_sites =
                                                !this.export_options.export_sites;
                                            cx.notify();
                                        }),
                                    )
                                    .child(self.render_checkbox_mark(opt_sites))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_primary())
                                            .child(format!(
                                                "{} ({} serverů)",
                                                t.export_opt_sites(),
                                                self.saved_sites.len()
                                            )),
                                    ),
                            )
                            // Option 2: Passwords (sub-checkbox)
                            .when(opt_sites, |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .pl_5()
                                        .gap_1()
                                        .child(
                                            div()
                                                .id("cb-export-pwd")
                                                .cursor_pointer()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(|this, _, _, cx| {
                                                        this.export_options.export_passwords =
                                                            !this.export_options.export_passwords;
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(self.render_checkbox_mark(opt_pwd))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme.text_primary())
                                                        .child(t.export_opt_passwords()),
                                                ),
                                        )
                                        .when(opt_pwd, |sub| {
                                            sub.child(
                                                div()
                                                    .text_size(px(10.0))
                                                    .text_color(rgb(0xd97706))
                                                    .child(t.export_passwords_warning()),
                                            )
                                        }),
                                )
                            })
                            // Option 3: Settings
                            .child(
                                div()
                                    .id("cb-export-settings")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.export_options.export_settings =
                                                !this.export_options.export_settings;
                                            cx.notify();
                                        }),
                                    )
                                    .child(self.render_checkbox_mark(opt_settings))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_primary())
                                            .child(t.export_opt_settings()),
                                    ),
                            )
                            // Option 4: Favorites
                            .child(
                                div()
                                    .id("cb-export-favs")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.export_options.export_favorites =
                                                !this.export_options.export_favorites;
                                            cx.notify();
                                        }),
                                    )
                                    .child(self.render_checkbox_mark(opt_favs))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_primary())
                                            .child(format!(
                                                "{} ({} složek)",
                                                t.export_opt_favorites(),
                                                self.favorite_paths.len()
                                            )),
                                    ),
                            ),
                    )
                    // Export Status Banner
                    .when_some(self.export_status.clone(), |el, status| {
                        el.child(
                            div()
                                .p_2p5()
                                .rounded(px(6.0))
                                .bg(rgba(0x10b98115))
                                .border_1()
                                .border_color(rgba(0x10b98140))
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(render_icon("icons/check.svg", 14.0, 0x10b981))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0x10b981))
                                        .child(status),
                                ),
                        )
                    })
                    // Bottom Buttons
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .pt_1()
                            .child(
                                div()
                                    .id("btn-cancel-export")
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
                                            this.active_modal = ActiveModal::None;
                                            cx.notify();
                                        }),
                                    )
                                    .child(t.btn_cancel()),
                            )
                            .child(
                                div()
                                    .id("btn-perform-export")
                                    .cursor_pointer()
                                    .px_4()
                                    .py_1p5()
                                    .rounded(px(5.0))
                                    .bg(rgb(0x2563eb))
                                    .hover(|s| s.bg(rgb(0x1d4ed8)))
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xffffff))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.execute_export(cx);
                                        }),
                                    )
                                    .child(t.btn_export_action()),
                            ),
                    ),
            )
    }

    pub fn execute_export(&mut self, cx: &mut Context<Self>) {
        let now = chrono_format_date();
        let default_name = format!("ruster-backup-{}.json", now);
        let initial_dir = self.local_dir.clone();
        let rx = cx.prompt_for_new_path(&initial_dir, Some(&default_name));

        cx.spawn(move |this: gpui::WeakEntity<RusterView>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                if let Ok(Ok(Some(path))) = rx.await {
                    let _ = this.update(&mut cx, |view, cx| {
                        let state = crate::config::AppState {
                            show_local_pane: view.show_local_pane,
                            show_hidden: view.show_hidden,
                            show_function_bar: view.show_function_bar,
                            show_permissions: view.show_permissions,
                            date_format: view.date_format,
                            local_dir: view.local_dir.clone(),
                            right_dir: view.right_dir.clone(),
                            max_concurrent_transfers: view.max_concurrent_transfers,
                            editable_extensions: view.editable_extensions.clone(),
                            window_width: 1280.0,
                            window_height: 800.0,
                            queue_panel_height: view.queue_panel_height,
                            theme: view.theme,
                            locale: view.locale,
                        };

                        match create_ruster_backup(
                            &state,
                            &view.saved_sites,
                            &view.favorite_paths,
                            &view.export_options,
                        ) {
                            Ok(json) => {
                                if let Err(e) = fs::write(&path, json) {
                                    view.export_status = Some(format!("Chyba při zápisu: {}", e));
                                } else {
                                    view.export_status = Some(format!(
                                        "Záloha byla úspěšně uložena do: {}",
                                        path.file_name().and_then(|n| n.to_str()).unwrap_or("souboru")
                                    ));
                                }
                            }
                            Err(e) => {
                                view.export_status = Some(format!("Chyba: {}", e));
                            }
                        }
                        cx.notify();
                    });
                }
            }
        })
        .detach();
    }
}

fn chrono_format_date() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple date calculation YYYY-MM-DD
    let days = secs / 86400;
    // Approximating date or epoch seconds
    format!("{}", days)
}
