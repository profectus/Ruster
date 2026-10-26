use gpui::{div, prelude::*, px, rgb, Context, IntoElement, MouseButton};
use crate::app::RusterView;
use crate::icons::render_icon;
use crate::i18n::Locale;
use crate::model::{mode_to_rwx, ActiveModal};

impl RusterView {
    pub fn render_chmod_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let mode = self.chmod_mode;
        let rwx_preview = mode_to_rwx(mode, false, false);

        let bit_buttons: [(&str, u32); 9] = match self.locale {
            Locale::Cs => [
                ("Vlastník — Čtení (r)", 0o400),
                ("Vlastník — Zápis (w)", 0o200),
                ("Vlastník — Spuštění (x)", 0o100),
                ("Skupina — Čtení (r)", 0o040),
                ("Skupina — Zápis (w)", 0o020),
                ("Skupina — Spuštění (x)", 0o010),
                ("Ostatní — Čtení (r)", 0o004),
                ("Ostatní — Zápis (w)", 0o002),
                ("Ostatní — Spuštění (x)", 0o001),
            ],
            Locale::De => [
                ("Besitzer — Lesen (r)", 0o400),
                ("Besitzer — Schreiben (w)", 0o200),
                ("Besitzer — Ausführen (x)", 0o100),
                ("Gruppe — Lesen (r)", 0o040),
                ("Gruppe — Schreiben (w)", 0o020),
                ("Gruppe — Ausführen (x)", 0o010),
                ("Andere — Lesen (r)", 0o004),
                ("Andere — Schreiben (w)", 0o002),
                ("Andere — Ausführen (x)", 0o001),
            ],
            Locale::Es => [
                ("Propietario — Lectura (r)", 0o400),
                ("Propietario — Escritura (w)", 0o200),
                ("Propietario — Ejecución (x)", 0o100),
                ("Grupo — Lectura (r)", 0o040),
                ("Grupo — Escritura (w)", 0o020),
                ("Grupo — Ejecución (x)", 0o010),
                ("Otros — Lectura (r)", 0o004),
                ("Otros — Escritura (w)", 0o002),
                ("Otros — Ejecución (x)", 0o001),
            ],
            Locale::Fr => [
                ("Propriétaire — Lecture (r)", 0o400),
                ("Propriétaire — Écriture (w)", 0o200),
                ("Propriétaire — Exécution (x)", 0o100),
                ("Groupe — Lecture (r)", 0o040),
                ("Groupe — Écriture (w)", 0o020),
                ("Groupe — Exécution (x)", 0o010),
                ("Autres — Lecture (r)", 0o004),
                ("Autres — Écriture (w)", 0o002),
                ("Autres — Exécution (x)", 0o001),
            ],
            Locale::It => [
                ("Proprietario — Lettura (r)", 0o400),
                ("Proprietario — Scrittura (w)", 0o200),
                ("Proprietario — Esecuzione (x)", 0o100),
                ("Gruppo — Lettura (r)", 0o040),
                ("Gruppo — Scrittura (w)", 0o020),
                ("Gruppo — Esecuzione (x)", 0o010),
                ("Altri — Lettura (r)", 0o004),
                ("Altri — Scrittura (w)", 0o002),
                ("Altri — Esecuzione (x)", 0o001),
            ],
            Locale::Pl => [
                ("Właściciel — Odczyt (r)", 0o400),
                ("Właściciel — Zapis (w)", 0o200),
                ("Właściciel — Wykonanie (x)", 0o100),
                ("Grupa — Odczyt (r)", 0o040),
                ("Grupa — Zapis (w)", 0o020),
                ("Grupa — Wykonanie (x)", 0o010),
                ("Inni — Odczyt (r)", 0o004),
                ("Inni — Zapis (w)", 0o002),
                ("Inni — Wykonanie (x)", 0o001),
            ],
            Locale::Pt => [
                ("Proprietário — Leitura (r)", 0o400),
                ("Proprietário — Escrita (w)", 0o200),
                ("Proprietário — Execução (x)", 0o100),
                ("Grupo — Leitura (r)", 0o040),
                ("Grupo — Escrita (w)", 0o020),
                ("Grupo — Execução (x)", 0o010),
                ("Outros — Leitura (r)", 0o004),
                ("Outros — Escrita (w)", 0o002),
                ("Outros — Execução (x)", 0o001),
            ],
            Locale::Zh => [
                ("所有者 — 读取 (r)", 0o400),
                ("所有者 — 写入 (w)", 0o200),
                ("所有者 — 执行 (x)", 0o100),
                ("用户组 — 读取 (r)", 0o040),
                ("用户组 — 写入 (w)", 0o020),
                ("用户组 — 执行 (x)", 0o010),
                ("其他用户 — 读取 (r)", 0o004),
                ("其他用户 — 写入 (w)", 0o002),
                ("其他用户 — 执行 (x)", 0o001),
            ],
            Locale::Ja => [
                ("所有者 — 読み取り (r)", 0o400),
                ("所有者 — 書き込み (w)", 0o200),
                ("所有者 — 実行 (x)", 0o100),
                ("グループ — 読み取り (r)", 0o040),
                ("グループ — 書き込み (w)", 0o020),
                ("グループ — 実行 (x)", 0o010),
                ("その他 — 読み取り (r)", 0o004),
                ("その他 — 書き込み (w)", 0o002),
                ("その他 — 実行 (x)", 0o001),
            ],
            _ => [
                ("Owner — Read (r)", 0o400),
                ("Owner — Write (w)", 0o200),
                ("Owner — Execute (x)", 0o100),
                ("Group — Read (r)", 0o040),
                ("Group — Write (w)", 0o020),
                ("Group — Execute (x)", 0o010),
                ("Others — Read (r)", 0o004),
                ("Others — Write (w)", 0o002),
                ("Others — Execute (x)", 0o001),
            ],
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
                    .w(px(440.0))
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
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(render_icon("icons/shield.svg", 14.0, theme.c_icon_dark()))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(t.modal_chmod_title()),
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
                                    .text_color(theme.text_primary())
                                    .child(format!("{:04o} ({})", mode, rwx_preview)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .children(bit_buttons.into_iter().enumerate().map(
                                |(idx, (label, bit))| {
                                    let is_on = (mode & bit) != 0;
                                    div()
                                        .id(("chmod-bit", idx))
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px_3()
                                        .py_1()
                                        .rounded(px(4.0))
                                        .bg(if is_on {
                                            theme.button_active_bg()
                                        } else {
                                            theme.modal_card_bg()
                                        })
                                        .border_1()
                                        .border_color(if is_on {
                                            theme.button_active_border()
                                        } else {
                                            theme.border()
                                        })
                                        .hover(|s| s.bg(theme.button_hover_bg()))
                                        .text_xs()
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.chmod_mode ^= bit;
                                                cx.notify();
                                            }),
                                        )
                                        .child(div().text_color(theme.text_primary()).child(label))
                                        .child(
                                            div()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(if is_on {
                                                    theme.text_accent()
                                                } else {
                                                    theme.text_muted()
                                                })
                                                .child(t.chmod_enabled(is_on)),
                                        )
                                },
                            )),
                    )
                    .child(
                        div()
                            .id("chmod-recursive-toggle")
                            .cursor_pointer()
                            .px_3()
                            .py_1p5()
                            .rounded(px(4.0))
                            .bg(theme.modal_card_bg())
                            .border_1()
                            .border_color(theme.border())
                            .hover(|s| s.bg(theme.button_hover_bg()))
                            .text_xs()
                            .text_color(theme.text_primary())
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.chmod_recursive = !this.chmod_recursive;
                                    cx.notify();
                                }),
                            )
                            .child(t.chmod_recursive_label(self.chmod_recursive)),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                div()
                                    .id("chmod-cancel")
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
                                    .id("chmod-save")
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
                                            this.apply_chmod();
                                            cx.notify();
                                        }),
                                    )
                                    .child(t.btn_save()),
                            ),
                    ),
            )
    }
}
