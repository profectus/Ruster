use gpui::{
    div, prelude::*, px, rgb, App, Context, IntoElement, Menu, MenuItem, MouseButton,
};
use std::path::PathBuf;
use crate::app::{ContextMenuState, RusterView};
use crate::icons::render_icon;
use crate::i18n::{I18n, Locale};
use crate::model::ActivePane;
use crate::{
    CheckForUpdates, CompressModal, CopySelected, DeleteModal, EditSelected, ExtractModal,
    FileCompareModal, FindFilesModal, GoToFolderModal, MoveSelected, MultiRenameModal,
    NewFolderModal, OpenInTerminal, OpenSelected, OpenSiteManager, PermissionsModal, Quit, RefreshPanes,
    RenameModal, RevealInFinder, SelectAllItems, SwapPanes, SwitchLanguage, SynchronizeModal,
    ToggleDualPane, ToggleFunctionBar, ToggleHiddenFiles, ToggleLocalTree, ToggleQuickFilter, ToggleRemoteTree,
    ToggleTheme, ToggleTransferQueue,
};

impl RusterView {
    pub fn render_session_dropdown_popover(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        div()
            .absolute()
            .top(px(36.0))
            .left(px(12.0))
            .w(px(310.0))
            .rounded(px(6.0))
            .bg(theme.popover_bg())
            .border_1()
            .border_color(theme.border_strong())
            .shadow_lg()
            .p_1()
            .flex()
            .flex_col()
            .gap_0p5()
            .children(self.sessions.iter().enumerate().map(|(idx, session)| {
                let is_active = idx == self.active_session_idx;
                let label = format!("{}@{}", session.username, session.host);
                let proto = session.protocol.short_badge();
                let color = session.env_color;

                div()
                    .id(("dropdown-session-item", idx))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_2p5()
                    .py_1p5()
                    .rounded(px(4.0))
                    .bg(if is_active {
                        theme.popover_item_active()
                    } else {
                        theme.popover_bg()
                    })
                    .hover(|s| s.bg(theme.popover_item_active()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            if this.active_session_idx != idx {
                                if let Some(s) = this.sessions.get(idx) {
                                    this.trigger_remote_flash(s.env_color, false, cx);
                                }
                            }
                            this.active_session_idx = idx;
                            this.session_dropdown_open = false;
                            this.active_pane = ActivePane::Remote;
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(6.0))
                                    .h(px(6.0))
                                    .rounded(px(3.0))
                                    .bg(rgb(color)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(if is_active {
                                        gpui::FontWeight::SEMIBOLD
                                    } else {
                                        gpui::FontWeight::NORMAL
                                    })
                                    .text_color(theme.text_primary())
                                    .child(label),
                            ),
                    )
                    .child(div().text_xs().text_color(theme.text_secondary()).child(proto))
            }))
            .child(div().h(px(1.0)).my_1().bg(theme.border()))
            .child(
                div()
                    .id("dropdown-new-conn")
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2p5()
                    .py_1p5()
                    .rounded(px(4.0))
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.text_accent())
                    .hover(|s| s.bg(theme.popover_item_active()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.open_site_manager();
                            cx.notify();
                        }),
                    )
                    .child(render_icon("icons/server.svg", 13.0, theme.c_icon_active()))
                    .child(t.menu_site_manager()),
            )
    }

    pub fn render_favorites_popover(
        &self,
        pane: ActivePane,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let home_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users"));

        let current_dir = match pane {
            ActivePane::Local => self.local_dir.clone(),
            ActivePane::Remote => self.right_dir.clone(),
        };
        let is_already_favorite = self.favorite_paths.contains(&current_dir);

        div()
            .id(if pane == ActivePane::Local {
                "favorites-popover-local"
            } else {
                "favorites-popover-right"
            })
            .absolute()
            .top(px(34.0))
            .right(px(10.0))
            .w(px(280.0))
            .rounded(px(6.0))
            .bg(theme.popover_bg())
            .border_1()
            .border_color(theme.border_strong())
            .shadow_lg()
            .p_1()
            .flex()
            .flex_col()
            .gap_0p5()
            .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.stop_propagation()))
            .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
            // Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_2()
                    .py_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .child(render_icon("icons/star.svg", 13.0, theme.c_icon_active()))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text_primary())
                                    .child(t.favorites_title()),
                            ),
                    )
                    .child(
                        div()
                            .id("fav-header-close")
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(18.0))
                            .h(px(18.0))
                            .rounded(px(3.0))
                            .hover(|s| s.bg(theme.button_hover_bg()))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.favorites_menu_pane = None;
                                    cx.notify();
                                }),
                            )
                            .child(render_icon("icons/close.svg", 11.0, theme.c_icon_dark())),
                    ),
            )
            .child(div().h(px(1.0)).my_0p5().bg(theme.border()))
            // List of favorites
            .child(
                div()
                    .id("fav-list-scroll")
                    .max_h(px(240.0))
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .when(self.favorite_paths.is_empty(), |el| {
                        el.child(
                            div()
                                .px_2p5()
                                .py_2()
                                .text_xs()
                                .text_color(theme.text_secondary())
                                .child(t.favorites_empty()),
                        )
                    })
                    .children(self.favorite_paths.iter().enumerate().map(|(idx, fav_path)| {
                        let path_clone = fav_path.clone();
                        let path_for_remove = fav_path.clone();
                        let is_current = &current_dir == fav_path;

                        let display_path = if fav_path == &home_dir {
                            "~".to_string()
                        } else if let Ok(rel) = fav_path.strip_prefix(&home_dir) {
                            format!("~/{}", rel.display())
                        } else {
                            fav_path.to_string_lossy().to_string()
                        };

                        div()
                            .id(("fav-item", idx))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_1p5()
                            .rounded(px(4.0))
                            .bg(if is_current {
                                theme.popover_item_active()
                            } else {
                                theme.popover_bg()
                            })
                            .hover(|s| s.bg(theme.popover_item_active()))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    match pane {
                                        ActivePane::Local => this.navigate_local(path_clone.clone()),
                                        ActivePane::Remote => this.navigate_right_local(path_clone.clone()),
                                    }
                                    this.favorites_menu_pane = None;
                                    cx.notify();
                                }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .overflow_hidden()
                                    .child(render_icon("icons/folder.svg", 14.0, theme.c_folder()))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(if is_current {
                                                gpui::FontWeight::SEMIBOLD
                                            } else {
                                                gpui::FontWeight::NORMAL
                                            })
                                            .text_color(if is_current {
                                                theme.text_accent()
                                            } else {
                                                theme.text_primary()
                                            })
                                            .overflow_hidden()
                                            .whitespace_nowrap()
                                            .child(display_path),
                                    ),
                            )
                            .child(
                                div()
                                    .id(("fav-del", idx))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .rounded(px(3.0))
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            cx.stop_propagation();
                                            this.remove_favorite_path_by_value(&path_for_remove);
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/trash.svg", 12.0, theme.c_icon_secondary())),
                            )
                    })),
            )
            .child(div().h(px(1.0)).my_0p5().bg(theme.border()))
            // Add current location
            .child(
                if is_already_favorite {
                    div()
                        .id("fav-already-added")
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded(px(4.0))
                        .text_xs()
                        .text_color(theme.text_secondary())
                        .child(render_icon("icons/check.svg", 12.0, 0x16a34a))
                        .child(t.favorites_already_added())
                } else {
                    let cur_to_add = current_dir.clone();
                    div()
                        .id("fav-add-current")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded(px(4.0))
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.text_accent())
                        .hover(|s| s.bg(theme.popover_item_active()))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.add_favorite_path(cur_to_add.clone());
                                cx.notify();
                            }),
                        )
                        .child(render_icon("icons/plus.svg", 12.0, theme.c_icon_active()))
                        .child(t.favorites_add_current())
                },
            )
    }

    pub fn render_context_menu(
        &self,
        menu: &ContextMenuState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let x = menu.position.x.min(px(1040.0));
        let y = menu.position.y.min(px(580.0));
        let is_local = menu.pane == ActivePane::Local || (menu.pane == ActivePane::Remote && self.sessions.is_empty());
        let is_dir = match menu.pane {
            ActivePane::Local => self.local_entries.get(self.local_selected).map(|e| e.is_dir).unwrap_or(false),
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    self.right_entries.get(self.right_selected).map(|e| e.is_dir).unwrap_or(false)
                } else if let Some(s) = self.sessions.get(self.active_session_idx) {
                    s.entries.get(s.selected_index).map(|e| e.is_dir).unwrap_or(false)
                } else {
                    false
                }
            }
        };
        let is_archive = match menu.pane {
            ActivePane::Local => self.local_entries.get(self.local_selected).map(|e| crate::archive::is_archive_file(&e.name)).unwrap_or(false),
            ActivePane::Remote => {
                if self.sessions.is_empty() {
                    self.right_entries.get(self.right_selected).map(|e| crate::archive::is_archive_file(&e.name)).unwrap_or(false)
                } else if let Some(s) = self.sessions.get(self.active_session_idx) {
                    s.entries.get(s.selected_index).map(|e| crate::archive::is_archive_file(&e.name)).unwrap_or(false)
                } else {
                    false
                }
            }
        };

        div()
            .absolute()
            .top(y)
            .left(x)
            .w(px(220.0))
            .rounded(px(6.0))
            .bg(theme.popover_bg())
            .border_1()
            .border_color(theme.border_strong())
            .shadow_lg()
            .p_1()
            .flex()
            .flex_col()
            .gap_0p5()
            .text_xs()
            .child(self.render_ctx_item("ctx-open", t.act_open(), cx, |this, cx| {
                this.context_menu = None;
                this.open_selected_item(cx);
                cx.notify();
            }))
            .when(!is_dir, |el| {
                el.child(self.render_ctx_item("ctx-edit-in-editor", t.act_edit_in_editor(), cx, |this, cx| {
                    this.context_menu = None;
                    this.open_selected_item(cx);
                    cx.notify();
                }))
                .child(self.render_ctx_item("ctx-open-with", t.act_open_in_app(), cx, |this, cx| {
                    this.context_menu = None;
                    this.open_selected_item_with_app(cx);
                    cx.notify();
                }))
            })
            .child(self.render_ctx_item("ctx-copy", t.act_copy(), cx, |this, cx| {
                this.context_menu = None;
                this.copy_selected_item(false, cx);
                cx.notify();
            }))
            .child(self.render_ctx_item("ctx-move", t.act_move(), cx, |this, cx| {
                this.context_menu = None;
                this.copy_selected_item(true, cx);
                cx.notify();
            }))
            .child(div().h(px(1.0)).my_0p5().bg(theme.border()))
            .child(self.render_ctx_item(
                "ctx-mkdir",
                t.act_new_folder(),
                cx,
                |this, cx| {
                    this.context_menu = None;
                    this.open_new_folder_modal();
                    cx.notify();
                },
            ))
            .child(self.render_ctx_item("ctx-rename", t.act_rename(), cx, |this, cx| {
                this.context_menu = None;
                this.open_rename_modal();
                cx.notify();
            }))
            .child(self.render_ctx_item("ctx-multirename", t.menu_multirename(), cx, |this, cx| {
                this.context_menu = None;
                this.open_multirename_modal();
                cx.notify();
            }))
            .child(self.render_ctx_item(
                "ctx-chmod",
                t.act_permissions(),
                cx,
                |this, cx| {
                    this.context_menu = None;
                    this.open_chmod_modal();
                    cx.notify();
                },
            ))
            .child(self.render_ctx_item("ctx-delete", t.act_delete(), cx, |this, cx| {
                this.context_menu = None;
                this.open_delete_modal();
                cx.notify();
            }))
            .child(div().h(px(1.0)).my_0p5().bg(theme.border()))
            .child(self.render_ctx_item("ctx-compress", t.act_compress(), cx, |this, cx| {
                this.context_menu = None;
                this.open_compress_modal();
                cx.notify();
            }))
            .when(is_archive, |el| {
                el.child(self.render_ctx_item("ctx-extract", t.act_extract(), cx, |this, cx| {
                    this.context_menu = None;
                    this.open_extract_modal();
                    cx.notify();
                }))
            })
            .when(is_local, |el| {
                el.child(div().h(px(1.0)).my_0p5().bg(theme.border()))
                    .child(
                        self.render_ctx_item(
                            "ctx-finder",
                            t.act_reveal_in_finder(),
                            cx,
                            |this, cx| {
                                this.context_menu = None;
                                this.reveal_selected_in_finder(cx);
                                cx.notify();
                            },
                        ),
                    )
                    .child(
                        self.render_ctx_item(
                            "ctx-terminal",
                            t.act_open_in_terminal(),
                            cx,
                            |this, cx| {
                                this.context_menu = None;
                                this.open_selected_in_terminal(cx);
                                cx.notify();
                            },
                        ),
                    )
            })
    }

    pub fn render_ctx_item(
        &self,
        id: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let theme = self.theme();
        div()
            .id(id)
            .cursor_pointer()
            .px_2p5()
            .py_1()
            .rounded(px(4.0))
            .text_color(theme.text_primary())
            .hover(|s| s.bg(theme.popover_item_active()))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    action(this, cx);
                }),
            )
            .child(label)
    }
}

pub fn install_app_menus(cx: &mut App, locale: Locale) {
    let t = I18n::new(locale);
    cx.set_menus(vec![
        Menu {
            name: "Ruster".into(),
            items: vec![
                MenuItem::action(t.menu_check_for_updates(), CheckForUpdates),
                MenuItem::separator(),
                MenuItem::action(t.menu_site_manager(), OpenSiteManager),
                MenuItem::separator(),
                MenuItem::action(t.menu_switch_language(), SwitchLanguage),
                MenuItem::separator(),
                MenuItem::action("Quit Ruster", Quit),
            ],
        },
        Menu {
            name: t.menu_connections().into(),
            items: vec![
                MenuItem::action(t.menu_site_manager(), OpenSiteManager),
                MenuItem::separator(),
                MenuItem::action(t.menu_refresh(), RefreshPanes),
            ],
        },
        Menu {
            name: t.menu_view().into(),
            items: vec![
                MenuItem::action(t.menu_toggle_dual_pane(), ToggleDualPane),
                MenuItem::action(t.menu_swap_panes(), SwapPanes),
                MenuItem::separator(),
                MenuItem::action(t.menu_toggle_local_tree(), ToggleLocalTree),
                MenuItem::action(t.menu_toggle_remote_tree(), ToggleRemoteTree),
                MenuItem::separator(),
                MenuItem::action(t.menu_toggle_hidden(), ToggleHiddenFiles),
                MenuItem::action(t.menu_toggle_queue(), ToggleTransferQueue),
                MenuItem::action(t.menu_toggle_function_bar(), ToggleFunctionBar),
                MenuItem::action(t.menu_quick_filter(), ToggleQuickFilter),
                MenuItem::separator(),
                MenuItem::action(t.menu_toggle_theme(), ToggleTheme),
                MenuItem::action(t.menu_refresh(), RefreshPanes),
                MenuItem::action(t.menu_switch_language(), SwitchLanguage),
            ],
        },
        Menu {
            name: t.menu_actions().into(),
            items: vec![
                MenuItem::action(t.act_open(), OpenSelected),
                MenuItem::action(t.act_edit(), EditSelected),
                MenuItem::separator(),
                MenuItem::action(t.act_copy(), CopySelected),
                MenuItem::action(t.act_move(), MoveSelected),
                MenuItem::separator(),
                MenuItem::action(t.act_new_folder(), NewFolderModal),
                MenuItem::action(t.act_rename(), RenameModal),
                MenuItem::action(t.menu_multirename(), MultiRenameModal),
                MenuItem::action(t.act_delete(), DeleteModal),
                MenuItem::separator(),
                MenuItem::action(t.menu_find_files(), FindFilesModal),
                MenuItem::action(t.menu_compare_files(), FileCompareModal),
                MenuItem::separator(),
                MenuItem::action(t.act_compress(), CompressModal),
                MenuItem::action(t.act_extract(), ExtractModal),
                MenuItem::separator(),
                MenuItem::action(t.act_permissions(), PermissionsModal),
                MenuItem::action(t.act_synchronize(), SynchronizeModal),
                MenuItem::separator(),
                MenuItem::action(t.act_reveal_in_finder(), RevealInFinder),
                MenuItem::action(t.act_open_in_terminal(), OpenInTerminal),
                MenuItem::action(t.act_select_all(), SelectAllItems),
                MenuItem::separator(),
                MenuItem::action(t.menu_go_to_folder(), GoToFolderModal),
            ],
        },
    ]);
}
