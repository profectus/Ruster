use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};
use crate::app::MacScpView;
use crate::icons::render_icon;
use crate::model::ActiveModal;
use crate::theme::ThemeMode;
use crate::ui::helpers::render_textarea_value_with_cursor;

impl MacScpView {
    pub fn render_settings_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let show_hidden = self.show_hidden;
        let is_dark = theme.is_dark();

        let header_title = match self.locale {
            crate::i18n::Locale::En => "MacSCP Settings",
            crate::i18n::Locale::Cs => "Nastavení MacSCP",
            crate::i18n::Locale::De => "MacSCP Einstellungen",
            crate::i18n::Locale::Es => "Configuración de MacSCP",
            crate::i18n::Locale::Fr => "Paramètres MacSCP",
            crate::i18n::Locale::It => "Impostazioni MacSCP",
            crate::i18n::Locale::Pl => "Ustawienia MacSCP",
            crate::i18n::Locale::Pt => "Configurações do MacSCP",
            crate::i18n::Locale::Zh => "MacSCP 设置",
            crate::i18n::Locale::Ja => "MacSCP 設定",
        };

        let btn_save_close = match self.locale {
            crate::i18n::Locale::En => "Save & Close",
            crate::i18n::Locale::Cs => "Uložit a zavřít",
            crate::i18n::Locale::De => "Speichern & Schließen",
            crate::i18n::Locale::Es => "Guardar y cerrar",
            crate::i18n::Locale::Fr => "Enregistrer et fermer",
            crate::i18n::Locale::It => "Salva e chiudi",
            crate::i18n::Locale::Pl => "Zapisz i zamknij",
            crate::i18n::Locale::Pt => "Salvar e fechar",
            crate::i18n::Locale::Zh => "保存并关闭",
            crate::i18n::Locale::Ja => "保存して閉じる",
        };

        let files_section_title = match self.locale {
            crate::i18n::Locale::En => "FILES & VISIBILITY",
            crate::i18n::Locale::Cs => "ZOBRAZENÍ A SOUBORY",
            crate::i18n::Locale::De => "DATEIEN & SICHTBARKEIT",
            crate::i18n::Locale::Es => "ARCHIVOS Y VISIBILIDAD",
            crate::i18n::Locale::Fr => "FICHIERS & VISIBILITÉ",
            crate::i18n::Locale::It => "FILE E VISIBILITÀ",
            crate::i18n::Locale::Pl => "PLIKI I WIDOCZNOŚĆ",
            crate::i18n::Locale::Pt => "ARQUIVOS E VISIBILIDADE",
            crate::i18n::Locale::Zh => "文件与可见性",
            crate::i18n::Locale::Ja => "ファイルと表示",
        };

        let show_hidden_title = match self.locale {
            crate::i18n::Locale::En => "Show hidden files (.dotfiles)",
            crate::i18n::Locale::Cs => "Zobrazovat skryté soubory (.dotfiles)",
            crate::i18n::Locale::De => "Versteckte Dateien anzeigen (.dotfiles)",
            crate::i18n::Locale::Es => "Mostrar archivos ocultos (.dotfiles)",
            crate::i18n::Locale::Fr => "Afficher les fichiers cachés (.dotfiles)",
            crate::i18n::Locale::It => "Mostra file nascosti (.dotfiles)",
            crate::i18n::Locale::Pl => "Pokazuj ukryte pliki (.dotfiles)",
            crate::i18n::Locale::Pt => "Mostrar arquivos ocultos (.dotfiles)",
            crate::i18n::Locale::Zh => "显示隐藏文件 (.dotfiles)",
            crate::i18n::Locale::Ja => "隠しファイルを表示 (.dotfiles)",
        };

        let show_hidden_desc = if show_hidden {
            match self.locale {
                crate::i18n::Locale::En => "Hidden files are currently visible",
                crate::i18n::Locale::Cs => "Skryté soubory jsou zobrazeny",
                crate::i18n::Locale::De => "Versteckte Dateien sind sichtbar",
                crate::i18n::Locale::Es => "Los archivos ocultos están visibles",
                crate::i18n::Locale::Fr => "Les fichiers cachés sont visibles",
                crate::i18n::Locale::It => "I file nascosti sono visibili",
                crate::i18n::Locale::Pl => "Ukryte pliki są widoczne",
                crate::i18n::Locale::Pt => "Arquivos ocultos estão visíveis",
                crate::i18n::Locale::Zh => "隐藏文件当前可见",
                crate::i18n::Locale::Ja => "隠しファイルが表示されています",
            }
        } else {
            match self.locale {
                crate::i18n::Locale::En => "Hidden files are currently hidden",
                crate::i18n::Locale::Cs => "Skryté soubory jsou skryty",
                crate::i18n::Locale::De => "Versteckte Dateien sind ausgeblendet",
                crate::i18n::Locale::Es => "Los archivos ocultos están ocultos",
                crate::i18n::Locale::Fr => "Les fichiers cachés sont masqués",
                crate::i18n::Locale::It => "I file nascosti sono nascosti",
                crate::i18n::Locale::Pl => "Ukryte pliki są schowane",
                crate::i18n::Locale::Pt => "Arquivos ocultos estão ocultos",
                crate::i18n::Locale::Zh => "隐藏文件当前已隐藏",
                crate::i18n::Locale::Ja => "隠しファイルは非表示です",
            }
        };

        let editing_section_title = match self.locale {
            crate::i18n::Locale::En => "REMOTE FILE EDITING (FTP / SFTP)",
            crate::i18n::Locale::Cs => "EDITACE TEXTOVÝCH SOUBORŮ (FTP / SFTP)",
            crate::i18n::Locale::De => "DATEIEN BEARBEITEN (FTP / SFTP)",
            crate::i18n::Locale::Es => "EDICIÓN DE ARCHIVOS REMOTOS (FTP / SFTP)",
            crate::i18n::Locale::Fr => "ÉDITION DE FICHIERS DISTANTS (FTP / SFTP)",
            crate::i18n::Locale::It => "MODIFICA FILE REMOTI (FTP / SFTP)",
            crate::i18n::Locale::Pl => "EDYCJA PLIKÓW ZDALNYCH (FTP / SFTP)",
            crate::i18n::Locale::Pt => "EDIÇÃO DE ARQUIVOS REMOTOS (FTP / SFTP)",
            crate::i18n::Locale::Zh => "远程文件编辑 (FTP / SFTP)",
            crate::i18n::Locale::Ja => "リモートファイル編集 (FTP / SFTP)",
        };

        let reset_defaults_label = match self.locale {
            crate::i18n::Locale::En => "Reset to defaults",
            crate::i18n::Locale::Cs => "Obnovit výchozí",
            crate::i18n::Locale::De => "Zurücksetzen",
            crate::i18n::Locale::Es => "Restablecer",
            crate::i18n::Locale::Fr => "Réinitialiser",
            crate::i18n::Locale::It => "Ripristina predefiniti",
            crate::i18n::Locale::Pl => "Przywróć domyślne",
            crate::i18n::Locale::Pt => "Redefinir padrões",
            crate::i18n::Locale::Zh => "恢复默认",
            crate::i18n::Locale::Ja => "初期値に戻す",
        };

        let exts_prompt = match self.locale {
            crate::i18n::Locale::En => "File extensions to open in external editor:",
            crate::i18n::Locale::Cs => "Přípony pro otevření v systémovém editoru:",
            crate::i18n::Locale::De => "Dateiendungen für externen Editor:",
            crate::i18n::Locale::Es => "Extensiones para abrir en editor externo:",
            crate::i18n::Locale::Fr => "Extensions à ouvrir dans l'éditeur externe :",
            crate::i18n::Locale::It => "Estensioni da aprire nell'editor esterno:",
            crate::i18n::Locale::Pl => "Rozszerzenia do otwierania w edytorze:",
            crate::i18n::Locale::Pt => "Extensões para abrir no editor externo:",
            crate::i18n::Locale::Zh => "在外部编辑器中打开的文件后缀：",
            crate::i18n::Locale::Ja => "外部エディタで開く拡張子：",
        };

        let exts_help = match self.locale {
            crate::i18n::Locale::En => "Double-clicking a remote file downloads it to a temporary folder and opens it in your default editor (VS Code, TextEdit, etc.). Saving automatically uploads it back.",
            crate::i18n::Locale::Cs => "Dvojklik na soubor na serveru jej stáhne do dočasné složky a otevře ve vašem výchozím editoru (VS Code, TextEdit atd.). Po uložení (Cmd+S) se soubor automaticky nahraje zpět na server.",
            crate::i18n::Locale::De => "Doppelklick lädt die Datei temporär herunter und öffnet sie in Ihrem Standardeditor. Nach dem Speichern wird sie automatisch hochgeladen.",
            crate::i18n::Locale::Es => "El doble clic descarga el archivo a una carpeta temporal y lo abre en su editor. Al guardar, se sube automáticamente.",
            crate::i18n::Locale::Fr => "Un double-clic télécharge le fichier temporairement et l'ouvre dans votre éditeur. L'enregistrement le renvoie automatiquement.",
            crate::i18n::Locale::It => "Il doppio clic scarica il file e lo apre nel tuo editor. Il salvataggio lo ricarica automaticamente.",
            crate::i18n::Locale::Pl => "Dwuklik pobiera plik do folderu tymczasowego i otwiera w edytorze. Zapis automatycznie wysyła plik na serwer.",
            crate::i18n::Locale::Pt => "O clique duplo baixa o arquivo temporariamente e abre no seu editor. Salvar faz o upload automático.",
            crate::i18n::Locale::Zh => "双击远程文件会下载到临时目录并使用默认编辑器打开。保存文件后会自动重新上传。",
            crate::i18n::Locale::Ja => "ダブルクリックでファイルを一時フォルダにダウンロードし、エディタで開きます。保存時に自動再アップロードされます。",
        };

        div()
            .id("modal-settings-backdrop")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .bg(theme.modal_backdrop())
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.settings_lang_dropdown_open = false;
                this.settings_editing_exts = false;
                this.active_modal = ActiveModal::None;
                cx.notify();
            }))
            .child(
                div()
                    .id("modal-settings-card")
                    .w(px(520.0))
                    .max_h(px(650.0))
                    .bg(theme.modal_bg())
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        cx.stop_propagation();
                        if this.settings_editing_exts {
                            this.settings_editing_exts = false;
                            cx.notify();
                        }
                    }))
                    // Header
                    .child(
                        div()
                            .flex()
                            .flex_shrink_0()
                            .items_center()
                            .justify_between()
                            .px_4()
                            .py_3()
                            .rounded_t(px(10.0))
                            .bg(theme.modal_header_bg())
                            .border_b_1()
                            .border_color(theme.border())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(render_icon("icons/settings.svg", 16.0, theme.c_icon_active()))
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_sm()
                                            .text_color(theme.text_primary())
                                            .child(header_title),
                                    ),
                            )
                            .child(
                                div()
                                    .id("btn-close-settings-x")
                                    .cursor_pointer()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.settings_lang_dropdown_open = false;
                                        this.settings_editing_exts = false;
                                        this.active_modal = ActiveModal::None;
                                        cx.notify();
                                    }))
                                    .child(render_icon("icons/close.svg", 13.0, theme.c_icon_default())),
                            ),
                    )
                    // Content Body
                    .child(
                        div()
                            .id("settings-modal-body")
                            .flex_1()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .overflow_y_scroll()
                            // Section: Language Selector Dropdown (Roletka)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                div()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_xs()
                                                    .text_color(theme.text_secondary())
                                                    .child(t.language_section_title()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.text_muted())
                                                    .child(t.language_desc()),
                                            ),
                                    )
                                    // Dropdown Trigger Button
                                    .child(
                                        div()
                                            .id("settings-language-dropdown-btn")
                                            .cursor_pointer()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_3()
                                            .py_2()
                                            .rounded(px(6.0))
                                            .bg(theme.modal_card_bg())
                                            .border_1()
                                            .border_color(if self.settings_lang_dropdown_open {
                                                theme.button_active_border()
                                            } else {
                                                theme.border()
                                            })
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.settings_lang_dropdown_open = !this.settings_lang_dropdown_open;
                                                cx.notify();
                                            }))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2p5()
                                                    .child(render_icon("icons/globe.svg", 15.0, theme.c_icon_active()))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .text_color(theme.text_primary())
                                                            .child(format!("{} {}", self.locale.flag(), self.locale.display_name())),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme.text_muted())
                                                            .child(format!("({})", self.locale.english_name())),
                                                    ),
                                            )
                                            .child(
                                                render_icon("icons/chevron-down.svg", 13.0, theme.c_icon_default())
                                            ),
                                    )
                                    // Dropdown Options Menu
                                    .when(self.settings_lang_dropdown_open, |el| {
                                        el.child(
                                            div()
                                                .id("settings-language-options-list")
                                                .flex()
                                                .flex_col()
                                                .rounded(px(6.0))
                                                .bg(theme.modal_card_bg())
                                                .border_1()
                                                .border_color(theme.border_strong())
                                                .p_1()
                                                .gap_0p5()
                                                .max_h(px(180.0))
                                                .overflow_y_scroll()
                                                .children(crate::i18n::Locale::ALL.iter().enumerate().map(|(idx, &loc)| {
                                                    let is_selected = loc == self.locale;
                                                    div()
                                                        .id(("settings-lang-opt", idx))
                                                        .cursor_pointer()
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .px_2p5()
                                                        .py_1p5()
                                                        .rounded(px(4.0))
                                                        .bg(if is_selected {
                                                            theme.button_active_bg()
                                                        } else {
                                                            theme.modal_card_bg()
                                                        })
                                                        .hover(|s| s.bg(theme.button_hover_bg()))
                                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                            this.set_locale(loc, cx);
                                                            this.settings_lang_dropdown_open = false;
                                                            cx.notify();
                                                        }))
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_2()
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .font_weight(if is_selected {
                                                                            gpui::FontWeight::SEMIBOLD
                                                                        } else {
                                                                            gpui::FontWeight::NORMAL
                                                                        })
                                                                        .text_color(if is_selected {
                                                                            theme.text_accent()
                                                                        } else {
                                                                            theme.text_primary()
                                                                        })
                                                                        .child(format!("{} {}", loc.flag(), loc.display_name())),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(theme.text_muted())
                                                                        .child(format!("({})", loc.english_name())),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .when(is_selected, |icon_el| {
                                                                    icon_el.child(render_icon("icons/check.svg", 13.0, theme.c_icon_active()))
                                                                })
                                                        )
                                                }))
                                        )
                                    }),
                            )
                            // Section: Appearance & Theme
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_xs()
                                            .text_color(theme.text_secondary())
                                            .child(t.theme_section_title()),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .gap_2()
                                            // Light Theme Option Card
                                            .child(
                                                div()
                                                    .id("theme-btn-light")
                                                    .cursor_pointer()
                                                    .flex_1()
                                                    .p_2p5()
                                                    .rounded(px(6.0))
                                                    .bg(if !is_dark { theme.button_active_bg() } else { theme.modal_card_bg() })
                                                    .border_1()
                                                    .border_color(if !is_dark { theme.button_active_border() } else { theme.border() })
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.set_theme(ThemeMode::Light, cx);
                                                    }))
                                                    .child(render_icon("icons/sun.svg", 15.0, if !is_dark { 0x2563eb } else { theme.c_icon_default() }))
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_col()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(if !is_dark { gpui::FontWeight::SEMIBOLD } else { gpui::FontWeight::NORMAL })
                                                                    .text_color(if !is_dark { theme.text_accent() } else { theme.text_primary() })
                                                                    .child(t.theme_light_label()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(theme.text_muted())
                                                                    .child(t.theme_light_desc()),
                                                            ),
                                                    ),
                                            )
                                            // Dark Theme Option Card
                                            .child(
                                                div()
                                                    .id("theme-btn-dark")
                                                    .cursor_pointer()
                                                    .flex_1()
                                                    .p_2p5()
                                                    .rounded(px(6.0))
                                                    .bg(if is_dark { theme.button_active_bg() } else { theme.modal_card_bg() })
                                                    .border_1()
                                                    .border_color(if is_dark { theme.button_active_border() } else { theme.border() })
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.set_theme(ThemeMode::Dark, cx);
                                                    }))
                                                    .child(render_icon("icons/moon.svg", 15.0, if is_dark { 0x60a5fa } else { theme.c_icon_default() }))
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_col()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(if is_dark { gpui::FontWeight::SEMIBOLD } else { gpui::FontWeight::NORMAL })
                                                                    .text_color(if is_dark { theme.text_accent() } else { theme.text_primary() })
                                                                    .child(t.theme_dark_label()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(theme.text_muted())
                                                                    .child(t.theme_dark_desc()),
                                                            ),
                                                    ),
                                            ),
                                    ),
                            )
                            // Section: Files & Visibility
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_xs()
                                            .text_color(theme.text_secondary())
                                            .child(files_section_title),
                                    )
                                    .child(
                                        div()
                                            .id("btn-toggle-show-hidden-settings")
                                            .cursor_pointer()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .p_2p5()
                                            .bg(theme.modal_card_bg())
                                            .rounded(px(6.0))
                                            .border_1()
                                            .border_color(theme.border())
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.show_hidden = !this.show_hidden;
                                                this.reload_local();
                                                this.reload_right();
                                                cx.notify();
                                            }))
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap_0p5()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .text_color(theme.text_primary())
                                                            .child(show_hidden_title),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme.text_secondary())
                                                            .child(show_hidden_desc),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .w(px(38.0))
                                                    .h(px(20.0))
                                                    .rounded(px(10.0))
                                                    .bg(if show_hidden { rgb(0x2563eb) } else if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) })
                                                    .p(px(2.0))
                                                    .flex()
                                                    .items_center()
                                                    .when(show_hidden, |el| el.justify_end())
                                                    .when(!show_hidden, |el| el.justify_start())
                                                    .child(
                                                        div()
                                                            .w(px(16.0))
                                                            .h(px(16.0))
                                                            .rounded(px(8.0))
                                                            .bg(rgb(0xffffff)),
                                                    ),
                                            ),
                                    ),
                            )
                            // Section: Remote Editing & File Types
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                div()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_xs()
                                                    .text_color(theme.text_secondary())
                                                    .child(editing_section_title),
                                            )
                                            .child(
                                                div()
                                                    .id("btn-reset-exts")
                                                    .cursor_pointer()
                                                    .text_xs()
                                                    .text_color(theme.text_accent())
                                                    .hover(|s| s.underline())
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        this.editable_extensions = crate::model::default_editable_extensions();
                                                        this.settings_cursor_pos = this.editable_extensions.chars().count();
                                                        cx.notify();
                                                    }))
                                                    .child(reset_defaults_label),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .p_2p5()
                                            .bg(theme.modal_card_bg())
                                            .rounded(px(6.0))
                                            .border_1()
                                            .border_color(theme.border())
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(theme.text_primary())
                                                    .child(exts_prompt),
                                            )
                                            .child(
                                                div()
                                                    .id("settings-exts-input")
                                                    .cursor_text()
                                                    .w_full()
                                                    .min_h(px(68.0))
                                                    .max_h(px(96.0))
                                                    .overflow_y_scroll()
                                                    .p_2()
                                                    .rounded(px(5.0))
                                                    .bg(theme.input_bg())
                                                    .border_1()
                                                    .border_color(if self.settings_editing_exts {
                                                        theme.input_border_focus()
                                                    } else {
                                                        theme.input_border()
                                                    })
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                        cx.stop_propagation();
                                                        this.settings_editing_exts = true;
                                                        this.settings_cursor_pos = this.editable_extensions.chars().count();
                                                        cx.notify();
                                                    }))
                                                    .child(render_textarea_value_with_cursor(
                                                        &self.editable_extensions,
                                                        self.settings_editing_exts,
                                                        self.settings_cursor_pos,
                                                        "txt, md, json, php, js, html...",
                                                        is_dark,
                                                    )),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.text_secondary())
                                                    .child(exts_help),
                                            ),
                                    ),
                            )
                            // Section: Updates & About
                            .child(
                                div()
                                    .p_3()
                                    .rounded(px(6.0))
                                    .bg(theme.modal_card_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_0p5()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(theme.text_primary())
                                                    .child(format!("MacSCP v{}", env!("CARGO_PKG_VERSION"))),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.text_muted())
                                                    .child("GitHub: profectus/macSCP"),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .id("btn-settings-coffee")
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
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, _| {
                                                        let _ = std::process::Command::new("open")
                                                            .arg("https://buymeacoffee.com/filipstencl")
                                                            .spawn();
                                                    }))
                                                    .child("☕ Buy Me a Coffee"),
                                            )
                                            .child(
                                                div()
                                                    .id("btn-settings-check-updates")
                                                    .cursor_pointer()
                                                    .px_3()
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
                                                        this.check_for_updates(true, cx);
                                                    }))
                                                    .child(t.menu_check_for_updates()),
                                            ),
                                    ),
                            ),
                    )
                    // Footer
                    .child(
                        div()
                            .flex()
                            .flex_shrink_0()
                            .items_center()
                            .justify_end()
                            .gap_2()
                            .px_4()
                            .py_3()
                            .rounded_b(px(10.0))
                            .bg(theme.modal_header_bg())
                            .border_t_1()
                            .border_color(theme.border())
                            .child(
                                div()
                                    .id("btn-save-settings")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .px_3()
                                    .h(px(28.0))
                                    .rounded(px(5.0))
                                    .bg(rgb(0x2563eb))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(0xffffff))
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                        this.settings_lang_dropdown_open = false;
                                        this.settings_editing_exts = false;
                                        this.persist_app_state();
                                        this.active_modal = ActiveModal::None;
                                        cx.notify();
                                    }))
                                    .child(btn_save_close),
                            ),
                    ),
            )
    }
}
