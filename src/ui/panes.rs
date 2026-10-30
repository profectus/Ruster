use gpui::{
    div, prelude::*, px, rgb, rgba, uniform_list, Context, IntoElement,
    MouseButton, MouseDownEvent,
};
use std::ops::Range;
use std::path::PathBuf;
use crate::app::{ContextMenuState, DraggedFileItem, RusterView, ResizeColumnState};
use crate::icons::{get_file_icon, render_icon};
use crate::model::{sort_entries, ActiveModal, ActivePane, FileEntry, SortColumn};
use crate::ui::helpers::FileTooltip;

impl RusterView {
    pub fn render_local_pane(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_focused = self.active_pane == ActivePane::Local;
        let parent_dir = self.local_dir.parent().map(|p| p.to_path_buf());
        let home_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users"));

        let mut local_breadcrumbs: Vec<(String, PathBuf)> = Vec::new();
        let mut cur: Option<&std::path::Path> = Some(&self.local_dir);
        while let Some(p) = cur {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            local_breadcrumbs.push((name, p.to_path_buf()));
            cur = p.parent();
        }
        local_breadcrumbs.reverse();
        let crumb_count = local_breadcrumbs.len();

        let visible_indices = self.filtered_local_indices();
        let (total_dirs, total_files) = if self.show_local_filter && !self.local_filter_query.trim().is_empty() {
            let visible_entries: Vec<&FileEntry> = visible_indices.iter().filter_map(|&i| self.local_entries.get(i)).collect();
            let dirs = visible_entries.iter().filter(|e| e.is_dir && e.name != "..").count();
            let files = visible_entries.iter().filter(|e| !e.is_dir).count();
            (dirs, files)
        } else {
            let dirs = self
                .local_entries
                .iter()
                .filter(|e| e.is_dir && e.name != "..")
                .count();
            let files = self.local_entries.iter().filter(|e| !e.is_dir).count();
            (dirs, files)
        };
        let any_selected: Vec<&FileEntry> = self
            .local_entries
            .iter()
            .filter(|e| e.is_any_selected() && e.name != "..")
            .collect();
        let selected_count = any_selected.len();
        let selected_size: u64 = any_selected.iter().map(|e| e.effective_size()).sum();

        div()
            .id("pane-local")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.pane_bg())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.active_pane = ActivePane::Local;
                    this.session_dropdown_open = false;
                    this.context_menu = None;
                    this.favorites_menu_pane = None;
                    cx.notify();
                }),
            )
            // Breadcrumb & Tree Toggle Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(32.0))
                    .px_2p5()
                    .bg(if is_focused {
                        theme.header_focused_bg()
                    } else {
                        theme.header_unfocused_bg()
                    })
                    .border_b_1()
                    .border_color(theme.border())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .overflow_hidden()
                            .child(
                                div()
                                    .id("local-tree-toggle")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(if self.show_local_tree {
                                        theme.button_active_bg()
                                    } else {
                                        theme.button_secondary_bg()
                                    })
                                    .border_1()
                                    .border_color(if self.show_local_tree {
                                        theme.button_active_border()
                                    } else {
                                        theme.border()
                                    })
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.show_local_tree = !this.show_local_tree;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon(
                                        "icons/folder-tree.svg",
                                        13.0,
                                        if self.show_local_tree {
                                            theme.c_icon_active()
                                        } else {
                                            theme.c_icon_secondary()
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(if is_focused {
                                        theme.text_accent()
                                    } else {
                                        theme.text_muted()
                                    })
                                    .child(t.local_label()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_0p5()
                                    .children(local_breadcrumbs.into_iter().enumerate().map(
                                        |(i, (name, path))| {
                                            let is_last = i + 1 == crumb_count;
                                            div()
                                                .cursor_pointer()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded(px(3.0))
                                                .bg(if is_last {
                                                    theme.button_hover_bg()
                                                } else {
                                                    theme.button_secondary_bg()
                                                })
                                                .text_xs()
                                                .font_weight(if is_last {
                                                    gpui::FontWeight::SEMIBOLD
                                                } else {
                                                    gpui::FontWeight::NORMAL
                                                })
                                                .text_color(if is_last {
                                                    theme.text_primary()
                                                } else {
                                                    theme.text_secondary()
                                                })
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.navigate_local(path.clone());
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(name)
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .id("local-go-path-btn")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .rounded(px(3.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.open_go_to_folder_modal();
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/folder-input.svg", 12.0, theme.c_icon_secondary())),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .id("local-up")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            if let Some(ref p) = parent_dir {
                                                this.navigate_local(p.clone());
                                                cx.notify();
                                            }
                                        }),
                                    )
                                    .child(render_icon("icons/arrow-up.svg", 13.0, theme.c_icon_dark())),
                            )
                            .child(
                                div()
                                    .id("local-home")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.navigate_local(home_dir.clone());
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/home.svg", 13.0, theme.c_icon_dark())),
                            )
                            .child(
                                div()
                                    .id("local-favorites")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(if self.favorites_menu_pane == Some(ActivePane::Local) {
                                        theme.button_active_bg()
                                    } else {
                                        theme.button_secondary_bg()
                                    })
                                    .border_1()
                                    .border_color(if self.favorites_menu_pane == Some(ActivePane::Local) {
                                        theme.button_active_border()
                                    } else {
                                        theme.border()
                                    })
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            cx.stop_propagation();
                                            if this.favorites_menu_pane == Some(ActivePane::Local) {
                                                this.favorites_menu_pane = None;
                                            } else {
                                                this.favorites_menu_pane = Some(ActivePane::Local);
                                                this.session_dropdown_open = false;
                                                this.context_menu = None;
                                            }
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon(
                                        "icons/star.svg",
                                        13.0,
                                        if self.favorites_menu_pane == Some(ActivePane::Local) {
                                            theme.c_icon_active()
                                        } else {
                                            theme.c_icon_dark()
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .id("local-volumes")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.navigate_local(PathBuf::from("/Volumes"));
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/hard-drive.svg", 13.0, theme.c_icon_dark())),
                            ),
                    ),
            )
            .when(self.show_local_filter, |el| {
                el.child(self.render_pane_filter_bar(ActivePane::Local, cx))
            })
            // Main body: Optional scrollable Local Tree + Scrollable File Table
            .child(
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    .when(self.show_local_tree, |el| {
                        el.child(
                            div()
                                .id("local-tree-scroll")
                                .w(px(185.0))
                                .h_full()
                                .bg(theme.tree_bg())
                                .border_r_1()
                                .border_color(theme.border())
                                .p_1p5()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .overflow_y_scroll()
                                .children(self.local_tree.iter().enumerate().map(|(i, node)| {
                                    let target_path = node.path.clone();
                                    let indent = (node.depth.min(6) as f32) * 10.0;
                                    let is_cur = node.is_current;

                                    div()
                                        .id(("local-tree-node", i))
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .pl(px(6.0 + indent))
                                        .pr_1p5()
                                        .py_0p5()
                                        .rounded(px(4.0))
                                        .bg(if is_cur {
                                            theme.tree_node_active_bg()
                                        } else {
                                            theme.tree_node_normal_bg()
                                        })
                                        .text_xs()
                                        .font_weight(if is_cur {
                                            gpui::FontWeight::SEMIBOLD
                                        } else {
                                            gpui::FontWeight::NORMAL
                                        })
                                        .text_color(if is_cur {
                                            theme.row_text_dir()
                                        } else {
                                            theme.text_primary()
                                        })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.navigate_local(target_path.clone());
                                                cx.notify();
                                            }),
                                        )
                                        .child(render_icon(
                                            "icons/folder.svg",
                                            13.0,
                                            if is_cur { theme.c_icon_active() } else { theme.c_icon_default() },
                                        ))
                                        .child(node.label.clone())
                                })),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .h_full()
                            .overflow_hidden()
                            .child(self.render_table_header(true, cx))
                            .child(
                                div()
                                    .id("local-file-list-container")
                                    .flex_1()
                                    .h_full()
                                    .overflow_hidden()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            if this.active_modal != ActiveModal::None { return; }
                                            this.active_pane = ActivePane::Local;
                                            this.session_dropdown_open = false;
                                            this.context_menu = None;
                                            this.favorites_menu_pane = None;
                                            for e in &mut this.local_entries {
                                                e.selected = false;
                                                e.space_marked = false;
                                            }
                                            cx.notify();
                                        }),
                                    )
                                    .on_drop(cx.listener(|this, item: &DraggedFileItem, _, cx| {
                                        if item.source_pane != ActivePane::Local {
                                            this.copy_items(item.entries.clone(), item.source_pane, false, cx);
                                            cx.notify();
                                        }
                                    }))
                                    .child(
                                        uniform_list(
                                            "local-file-list",
                                            visible_indices.len(),
                                            cx.processor(move |this, range: Range<usize>, _window, cx| {
                                                let visible = this.filtered_local_indices();
                                                let mut items = Vec::with_capacity(range.len());
                                                for ix in range {
                                                    if let Some(&idx) = visible.get(ix) {
                                                        items.push(this.render_local_row(idx, cx));
                                                    }
                                                }
                                                items
                                            }),
                                        )
                                        .track_scroll(self.local_scroll.clone())
                                        .size_full(),
                                    ),
                            ),
                    ),
            )
            // Status footer
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(24.0))
                    .px_3()
                    .bg(theme.header_unfocused_bg())
                    .border_t_1()
                    .border_color(theme.border())
                    .text_xs()
                    .text_color(theme.text_secondary())
                    .child(t.status_counts(total_dirs, total_files, selected_count, selected_size)),
            )
            .when(self.favorites_menu_pane == Some(ActivePane::Local), |el| {
                el.child(self.render_favorites_popover(ActivePane::Local, cx))
            })
    }

    pub fn render_remote_pane(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let content = if self.sessions.is_empty() {
            self.render_right_local_pane(cx).into_any_element()
        } else {
            self.render_connected_remote_pane(cx).into_any_element()
        };

        let flash = self.remote_flash;

        div()
            .id("pane-remote-wrapper")
            .relative()
            .size_full()
            .overflow_hidden()
            .child(content)
            .when_some(flash, |el, f| {
                if f.opacity <= 0.005 {
                    return el;
                }
                let color = f.color;
                let (max_bg, max_border) = if f.is_disconnect {
                    (0.07, 0.50)
                } else {
                    (0.12, 0.85)
                };
                let bg_alpha = ((f.opacity * max_bg * 255.0) as u32).clamp(0, 255);
                let border_alpha = ((f.opacity * max_border * 255.0) as u32).clamp(0, 255);

                el.child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(rgba(((color & 0xffffff) << 8) | bg_alpha))
                        .border_2()
                        .border_color(rgba(((color & 0xffffff) << 8) | border_alpha)),
                )
            })
    }

    pub fn render_right_local_pane(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_focused = self.active_pane == ActivePane::Remote;
        let parent_dir = self.right_dir.parent().map(|p| p.to_path_buf());
        let home_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/Users"));

        let mut right_breadcrumbs: Vec<(String, PathBuf)> = Vec::new();
        let mut cur: Option<&std::path::Path> = Some(&self.right_dir);
        while let Some(p) = cur {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            right_breadcrumbs.push((name, p.to_path_buf()));
            cur = p.parent();
        }
        right_breadcrumbs.reverse();
        let crumb_count = right_breadcrumbs.len();

        let visible_indices = self.filtered_remote_indices();
        let (total_dirs, total_files) = if self.show_remote_filter && !self.remote_filter_query.trim().is_empty() {
            let visible_entries: Vec<&FileEntry> = visible_indices.iter().filter_map(|&i| self.right_entries.get(i)).collect();
            let dirs = visible_entries.iter().filter(|e| e.is_dir && e.name != "..").count();
            let files = visible_entries.iter().filter(|e| !e.is_dir).count();
            (dirs, files)
        } else {
            let dirs = self
                .right_entries
                .iter()
                .filter(|e| e.is_dir && e.name != "..")
                .count();
            let files = self.right_entries.iter().filter(|e| !e.is_dir).count();
            (dirs, files)
        };
        let any_selected: Vec<&FileEntry> = self
            .right_entries
            .iter()
            .filter(|e| e.is_any_selected() && e.name != "..")
            .collect();
        let selected_count = any_selected.len();
        let selected_size: u64 = any_selected.iter().map(|e| e.effective_size()).sum();

        div()
            .id("pane-right-local")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.pane_bg())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.active_pane = ActivePane::Remote;
                    this.session_dropdown_open = false;
                    this.context_menu = None;
                    this.favorites_menu_pane = None;
                    cx.notify();
                }),
            )
            // Breadcrumb & Tree Toggle Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(32.0))
                    .px_2p5()
                    .bg(if is_focused {
                        theme.header_focused_bg()
                    } else {
                        theme.header_unfocused_bg()
                    })
                    .border_b_1()
                    .border_color(theme.border())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .overflow_hidden()
                            .child(
                                div()
                                    .id("right-local-tree-toggle")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(if self.show_remote_tree {
                                        theme.button_active_bg()
                                    } else {
                                        theme.button_secondary_bg()
                                    })
                                    .border_1()
                                    .border_color(if self.show_remote_tree {
                                        theme.button_active_border()
                                    } else {
                                        theme.border()
                                    })
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.show_remote_tree = !this.show_remote_tree;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon(
                                        "icons/folder-tree.svg",
                                        13.0,
                                        if self.show_remote_tree {
                                            theme.c_icon_active()
                                        } else {
                                            theme.c_icon_secondary()
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(if is_focused {
                                        theme.text_accent()
                                    } else {
                                        theme.text_muted()
                                    })
                                    .child(t.local_label()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_0p5()
                                    .children(right_breadcrumbs.into_iter().enumerate().map(
                                        |(i, (name, path))| {
                                            let is_last = i + 1 == crumb_count;
                                            div()
                                                .cursor_pointer()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded(px(3.0))
                                                .bg(if is_last {
                                                    theme.button_hover_bg()
                                                } else {
                                                    theme.button_secondary_bg()
                                                })
                                                .text_xs()
                                                .font_weight(if is_last {
                                                    gpui::FontWeight::SEMIBOLD
                                                } else {
                                                    gpui::FontWeight::NORMAL
                                                })
                                                .text_color(if is_last {
                                                    theme.text_primary()
                                                } else {
                                                    theme.text_secondary()
                                                })
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.navigate_right_local(path.clone());
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(name)
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .id("right-local-go-path-btn")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .rounded(px(3.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.open_go_to_folder_modal();
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/folder-input.svg", 12.0, theme.c_icon_secondary())),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .id("right-local-up")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            if let Some(ref p) = parent_dir {
                                                this.navigate_right_local(p.clone());
                                                cx.notify();
                                            }
                                        }),
                                    )
                                    .child(render_icon("icons/arrow-up.svg", 13.0, theme.c_icon_dark())),
                            )
                            .child(
                                div()
                                    .id("right-local-home")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.navigate_right_local(home_dir.clone());
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/home.svg", 13.0, theme.c_icon_dark())),
                            )
                            .child(
                                div()
                                    .id("right-local-favorites")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(if self.favorites_menu_pane == Some(ActivePane::Remote) {
                                        theme.button_active_bg()
                                    } else {
                                        theme.button_secondary_bg()
                                    })
                                    .border_1()
                                    .border_color(if self.favorites_menu_pane == Some(ActivePane::Remote) {
                                        theme.button_active_border()
                                    } else {
                                        theme.border()
                                    })
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            cx.stop_propagation();
                                            if this.favorites_menu_pane == Some(ActivePane::Remote) {
                                                this.favorites_menu_pane = None;
                                            } else {
                                                this.favorites_menu_pane = Some(ActivePane::Remote);
                                                this.session_dropdown_open = false;
                                                this.context_menu = None;
                                            }
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon(
                                        "icons/star.svg",
                                        13.0,
                                        if self.favorites_menu_pane == Some(ActivePane::Remote) {
                                            theme.c_icon_active()
                                        } else {
                                            theme.c_icon_dark()
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .id("right-local-volumes")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.navigate_right_local(PathBuf::from("/Volumes"));
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/hard-drive.svg", 13.0, theme.c_icon_dark())),
                            ),
                    ),
            )
            .when(self.show_remote_filter, |el| {
                el.child(self.render_pane_filter_bar(ActivePane::Remote, cx))
            })
            // Main body: Optional scrollable Right Local Tree + Scrollable File Table
            .child(
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    .when(self.show_remote_tree, |el| {
                        el.child(
                            div()
                                .id("right-local-tree-scroll")
                                .w(px(185.0))
                                .h_full()
                                .bg(theme.tree_bg())
                                .border_r_1()
                                .border_color(theme.border())
                                .p_1p5()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .overflow_y_scroll()
                                .children(self.right_tree.iter().enumerate().map(|(i, node)| {
                                    let target_path = node.path.clone();
                                    let indent = (node.depth.min(6) as f32) * 10.0;
                                    let is_cur = node.is_current;

                                    div()
                                        .id(("right-local-tree-node", i))
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .pl(px(6.0 + indent))
                                        .pr_1p5()
                                        .py_0p5()
                                        .rounded(px(4.0))
                                        .bg(if is_cur {
                                            theme.tree_node_active_bg()
                                        } else {
                                            theme.tree_node_normal_bg()
                                        })
                                        .text_xs()
                                        .font_weight(if is_cur {
                                            gpui::FontWeight::SEMIBOLD
                                        } else {
                                            gpui::FontWeight::NORMAL
                                        })
                                        .text_color(if is_cur {
                                            theme.row_text_dir()
                                        } else {
                                            theme.text_primary()
                                        })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.navigate_right_local(target_path.clone());
                                                cx.notify();
                                            }),
                                        )
                                        .child(render_icon(
                                            "icons/folder.svg",
                                            13.0,
                                            if is_cur { theme.c_icon_active() } else { theme.c_icon_default() },
                                        ))
                                        .child(node.label.clone())
                                })),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .h_full()
                            .overflow_hidden()
                            .child(self.render_table_header(false, cx))
                            .child(
                                div()
                                    .id("right-local-file-list-container")
                                    .flex_1()
                                    .h_full()
                                    .overflow_hidden()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            if this.active_modal != ActiveModal::None { return; }
                                            this.active_pane = ActivePane::Remote;
                                            this.session_dropdown_open = false;
                                            this.context_menu = None;
                                            this.favorites_menu_pane = None;
                                            for e in &mut this.right_entries {
                                                e.selected = false;
                                                e.space_marked = false;
                                            }
                                            cx.notify();
                                        }),
                                    )
                                    .on_drop(cx.listener(|this, item: &DraggedFileItem, _, cx| {
                                        if item.source_pane != ActivePane::Remote {
                                            this.copy_items(item.entries.clone(), item.source_pane, false, cx);
                                            cx.notify();
                                        }
                                    }))
                                    .child(
                                        uniform_list(
                                            "right-local-file-list",
                                            visible_indices.len(),
                                            cx.processor(move |this, range: Range<usize>, _window, cx| {
                                                let visible = this.filtered_remote_indices();
                                                let mut items = Vec::with_capacity(range.len());
                                                for ix in range {
                                                    if let Some(&idx) = visible.get(ix) {
                                                        items.push(this.render_right_local_row(idx, cx));
                                                    }
                                                }
                                                items
                                            }),
                                        )
                                        .track_scroll(self.remote_scroll.clone())
                                        .size_full(),
                                    ),
                            ),
                    ),
            )
            // Status footer
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(24.0))
                    .px_3()
                    .bg(theme.header_unfocused_bg())
                    .border_t_1()
                    .border_color(theme.border())
                    .text_xs()
                    .text_color(theme.text_secondary())
                    .child(t.status_counts(total_dirs, total_files, selected_count, selected_size)),
            )
            .when(self.favorites_menu_pane == Some(ActivePane::Remote), |el| {
                el.child(self.render_favorites_popover(ActivePane::Remote, cx))
            })
    }

    pub fn render_connected_remote_pane(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_focused = self.active_pane == ActivePane::Remote;
        let session = match self.sessions.get(self.active_session_idx) {
            Some(s) => s,
            None => return div().id("pane-remote").size_full().into_any_element(),
        };
        let current_path = session.current_path.clone();
        let host_label = session.host.clone();

        let mut remote_breadcrumbs: Vec<(String, String)> = vec![("/".to_string(), "/".to_string())];
        let mut accum = String::new();
        for seg in current_path.split('/').filter(|s| !s.is_empty()) {
            accum.push('/');
            accum.push_str(seg);
            remote_breadcrumbs.push((seg.to_string(), accum.clone()));
        }
        let remote_crumb_count = remote_breadcrumbs.len();

        let mut remote_tree_nodes: Vec<(usize, String, String)> = Vec::new();
        remote_tree_nodes.push((0, "/".to_string(), "/".to_string()));
        let mut accum_tree = String::new();
        let segs: Vec<&str> = current_path.split('/').filter(|s| !s.is_empty()).collect();
        for (depth, seg) in segs.iter().enumerate() {
            accum_tree.push('/');
            accum_tree.push_str(seg);
            remote_tree_nodes.push((depth + 1, seg.to_string(), accum_tree.clone()));
        }
        let child_depth = segs.len() + 1;
        for entry in session.entries.iter().filter(|e| e.is_dir && e.name != "..") {
            let p = format!("{}/{}", current_path.trim_end_matches('/'), entry.name);
            remote_tree_nodes.push((child_depth, entry.name.clone(), p));
        }

        let visible_indices = self.filtered_remote_indices();
        let (total_dirs, total_files) = if self.show_remote_filter && !self.remote_filter_query.trim().is_empty() {
            let visible_entries: Vec<&FileEntry> = visible_indices.iter().filter_map(|&i| session.entries.get(i)).collect();
            let dirs = visible_entries.iter().filter(|e| e.is_dir && e.name != "..").count();
            let files = visible_entries.iter().filter(|e| !e.is_dir).count();
            (dirs, files)
        } else {
            let dirs = session
                .entries
                .iter()
                .filter(|e| e.is_dir && e.name != "..")
                .count();
            let files = session.entries.iter().filter(|e| !e.is_dir).count();
            (dirs, files)
        };
        let any_selected: Vec<&FileEntry> = session
            .entries
            .iter()
            .filter(|e| e.is_any_selected() && e.name != "..")
            .collect();
        let selected_count = any_selected.len();
        let selected_size: u64 = any_selected.iter().map(|e| e.effective_size()).sum();

        div()
            .id("pane-remote")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.pane_bg())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.active_pane = ActivePane::Remote;
                    this.session_dropdown_open = false;
                    this.context_menu = None;
                    cx.notify();
                }),
            )
            // Breadcrumb & Tree Toggle Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(32.0))
                    .px_2p5()
                    .bg(if is_focused {
                        theme.header_focused_bg()
                    } else {
                        theme.header_unfocused_bg()
                    })
                    .border_b_1()
                    .border_color(theme.border())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .overflow_hidden()
                            .child(
                                div()
                                    .id("remote-tree-toggle")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded(px(4.0))
                                    .bg(if self.show_remote_tree {
                                        theme.button_active_bg()
                                    } else {
                                        theme.button_secondary_bg()
                                    })
                                    .border_1()
                                    .border_color(if self.show_remote_tree {
                                        theme.button_active_border()
                                    } else {
                                        theme.border()
                                    })
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.show_remote_tree = !this.show_remote_tree;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon(
                                        "icons/folder-tree.svg",
                                        13.0,
                                        if self.show_remote_tree {
                                            theme.c_icon_active()
                                        } else {
                                            theme.c_icon_secondary()
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(
                                        div()
                                            .w(px(8.0))
                                            .h(px(8.0))
                                            .rounded(px(4.0))
                                            .flex_shrink_0()
                                            .bg(rgb(session.env_color)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(if is_focused {
                                                theme.text_accent()
                                            } else {
                                                theme.text_muted()
                                            })
                                            .child(host_label),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_0p5()
                                    .children(remote_breadcrumbs.into_iter().enumerate().map(
                                        |(i, (name, path))| {
                                            let is_last = i + 1 == remote_crumb_count;
                                            div()
                                                .cursor_pointer()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded(px(3.0))
                                                .bg(if is_last {
                                                    theme.button_hover_bg()
                                                } else {
                                                    theme.button_secondary_bg()
                                                })
                                                .text_xs()
                                                .font_weight(if is_last {
                                                    gpui::FontWeight::SEMIBOLD
                                                } else {
                                                    gpui::FontWeight::NORMAL
                                                })
                                                .text_color(if is_last {
                                                    theme.text_primary()
                                                } else {
                                                    theme.text_secondary()
                                                })
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.navigate_remote(path.clone());
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(name)
                                        },
                                    )),
                            )
                            .child(
                                div()
                                    .id("remote-go-path-btn")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .rounded(px(3.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.open_go_to_folder_modal();
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/folder-input.svg", 12.0, theme.c_icon_secondary())),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .id("rem-root")
                                    .cursor_pointer()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded(px(3.0))
                                    .bg(theme.button_secondary_bg())
                                    .border_1()
                                    .border_color(theme.border())
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .text_xs()
                                    .text_color(theme.text_primary())
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.navigate_remote("/".to_string());
                                            cx.notify();
                                        }),
                                    )
                                    .child("/"),
                            ),
                    ),
            )
            // Error banner if navigation or action failed
            .when_some(self.remote_error_banner.clone(), |el, err| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_1p5()
                        .bg(if theme.is_dark() { rgb(0x450a0a) } else { rgb(0xfef2f2) })
                        .border_b_1()
                        .border_color(if theme.is_dark() { rgb(0x7f1d1d) } else { rgb(0xfecaca) })
                        .text_xs()
                        .text_color(if theme.is_dark() { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                        .child(format!("⚠️ {}", err))
                        .child(
                            div()
                                .cursor_pointer()
                                .px_1()
                                .text_xs()
                                .font_weight(gpui::FontWeight::BOLD)
                                .child("×")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    this.remote_error_banner = None;
                                    cx.notify();
                                })),
                        ),
                )
            })
            .when(self.show_remote_filter, |el| {
                el.child(self.render_pane_filter_bar(ActivePane::Remote, cx))
            })
            // Main body: Optional Remote Tree + Scrollable Remote Table
            .child(
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    .when(self.show_remote_tree, |el| {
                        el.child(
                            div()
                                .id("remote-tree-scroll")
                                .w(px(185.0))
                                .h_full()
                                .bg(theme.tree_bg())
                                .border_r_1()
                                .border_color(theme.border())
                                .p_1p5()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .overflow_y_scroll()
                                .children(
                                    remote_tree_nodes
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, (depth, label, target))| {
                                            let target_str = target.clone();
                                            let indent = (depth.min(6) as f32) * 10.0;
                                            let is_cur = current_path == target;

                                            div()
                                                .id(("remote-tree-node", i))
                                                .cursor_pointer()
                                                .flex()
                                                .items_center()
                                                .gap_1p5()
                                                .pl(px(6.0 + indent))
                                                .pr_1p5()
                                                .py_0p5()
                                                .rounded(px(4.0))
                                                .bg(if is_cur {
                                                    theme.tree_node_active_bg()
                                                } else {
                                                    theme.tree_node_normal_bg()
                                                })
                                                .text_xs()
                                                .font_weight(if is_cur {
                                                    gpui::FontWeight::SEMIBOLD
                                                } else {
                                                    gpui::FontWeight::NORMAL
                                                })
                                                .text_color(if is_cur {
                                                    theme.row_text_dir()
                                                } else {
                                                    theme.text_primary()
                                                })
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.navigate_remote(target_str.clone());
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(render_icon(
                                                    "icons/folder.svg",
                                                    13.0,
                                                    if is_cur { theme.c_icon_active() } else { theme.c_icon_default() },
                                                ))
                                                .child(label)
                                        }),
                                ),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .h_full()
                            .overflow_hidden()
                            .child(self.render_table_header(false, cx))
                            .child(
                                div()
                                    .id("remote-file-list-container")
                                    .flex_1()
                                    .h_full()
                                    .overflow_hidden()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            if this.active_modal != ActiveModal::None { return; }
                                            this.active_pane = ActivePane::Remote;
                                            this.session_dropdown_open = false;
                                            this.context_menu = None;
                                            this.favorites_menu_pane = None;
                                            if let Some(s) = this.sessions.get_mut(this.active_session_idx) {
                                                for e in &mut s.entries {
                                                    e.selected = false;
                                                    e.space_marked = false;
                                                }
                                            }
                                            cx.notify();
                                        }),
                                    )
                                    .on_drop(cx.listener(|this, item: &DraggedFileItem, _, cx| {
                                        if item.source_pane != ActivePane::Remote {
                                            this.copy_items(item.entries.clone(), item.source_pane, false, cx);
                                            cx.notify();
                                        }
                                    }))
                                    .child(
                                        uniform_list(
                                            "remote-file-list",
                                            visible_indices.len(),
                                            cx.processor(move |this, range: Range<usize>, _window, cx| {
                                                let visible = this.filtered_remote_indices();
                                                let mut items = Vec::with_capacity(range.len());
                                                for ix in range {
                                                    if let Some(&idx) = visible.get(ix) {
                                                        items.push(this.render_remote_row(idx, cx));
                                                    }
                                                }
                                                items
                                            }),
                                        )
                                        .track_scroll(self.remote_scroll.clone())
                                        .size_full(),
                                    ),
                            ),
                    ),
            )
            // Status footer
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(24.0))
                    .px_3()
                    .bg(theme.header_unfocused_bg())
                    .border_t_1()
                    .border_color(theme.border())
                    .text_xs()
                    .text_color(theme.text_secondary())
                    .child(t.status_counts(total_dirs, total_files, selected_count, selected_size))
                    .child(format!(
                        "{}:{}",
                        session.protocol.short_badge(),
                        session.port
                    )),
            )
            .into_any_element()
    }

    pub fn render_col_resize_handle(&self, col_index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        div()
            .id(("col-resizer", col_index))
            .cursor_col_resize()
            .w(px(7.0))
            .flex_shrink_0()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .hover(move |s| s.bg(theme.border_strong()))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    this.resizing_column = Some(ResizeColumnState {
                        col_index,
                        start_mouse_x: f32::from(event.position.x),
                        initial_size_w: this.col_size_width,
                        initial_modified_w: this.col_modified_width,
                        initial_permissions_w: this.col_permissions_width,
                    });
                    cx.notify();
                }),
            )
            .child(
                div()
                    .w(px(1.0))
                    .h(px(12.0))
                    .bg(theme.border_strong()),
            )
    }

    pub fn render_table_header(&self, is_local: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let prefix = if is_local { "loc-sort" } else { "rem-sort" };

        div()
            .flex()
            .items_center()
            .h(px(24.0))
            .px_3()
            .bg(theme.table_header_bg())
            .border_b_1()
            .border_color(theme.border())
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(theme.text_secondary())
            .child(
                div()
                    .id((prefix, 0usize))
                    .cursor_pointer()
                    .flex_1()
                    .min_w_0()
                    .pr_3()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            if this.active_modal != ActiveModal::None { return; }
                            if is_local {
                                this.local_sort_asc = !this.local_sort_asc;
                                this.local_sort_col = SortColumn::Name;
                                sort_entries(
                                    &mut this.local_entries,
                                    this.local_sort_col,
                                    this.local_sort_asc,
                                );
                            } else if this.sessions.is_empty() {
                                this.remote_sort_asc = !this.remote_sort_asc;
                                this.remote_sort_col = SortColumn::Name;
                                sort_entries(
                                    &mut this.right_entries,
                                    this.remote_sort_col,
                                    this.remote_sort_asc,
                                );
                            } else if let Some(s) = this.sessions.get_mut(this.active_session_idx) {
                                this.remote_sort_asc = !this.remote_sort_asc;
                                this.remote_sort_col = SortColumn::Name;
                                sort_entries(
                                    &mut s.entries,
                                    this.remote_sort_col,
                                    this.remote_sort_asc,
                                );
                            }
                            cx.notify();
                        }),
                    )
                    .child(t.col_name()),
            )
            .child(self.render_col_resize_handle(0, cx))
            .child(
                div()
                    .id((prefix, 1usize))
                    .cursor_pointer()
                    .w(px(self.col_size_width))
                    .flex_shrink_0()
                    .px_2()
                    .text_right()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            if this.active_modal != ActiveModal::None { return; }
                            if is_local {
                                this.local_sort_asc = !this.local_sort_asc;
                                this.local_sort_col = SortColumn::Size;
                                sort_entries(
                                    &mut this.local_entries,
                                    this.local_sort_col,
                                    this.local_sort_asc,
                                );
                            } else if this.sessions.is_empty() {
                                this.remote_sort_asc = !this.remote_sort_asc;
                                this.remote_sort_col = SortColumn::Size;
                                sort_entries(
                                    &mut this.right_entries,
                                    this.remote_sort_col,
                                    this.remote_sort_asc,
                                );
                            } else if let Some(s) = this.sessions.get_mut(this.active_session_idx) {
                                this.remote_sort_asc = !this.remote_sort_asc;
                                this.remote_sort_col = SortColumn::Size;
                                sort_entries(
                                    &mut s.entries,
                                    this.remote_sort_col,
                                    this.remote_sort_asc,
                                );
                            }
                            cx.notify();
                        }),
                    )
                    .child(t.col_size()),
            )
            .child(self.render_col_resize_handle(1, cx))
            .child(
                div()
                    .id((prefix, 2usize))
                    .cursor_pointer()
                    .w(px(self.col_modified_width))
                    .flex_shrink_0()
                    .px_2()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            if this.active_modal != ActiveModal::None { return; }
                            if is_local {
                                this.local_sort_asc = !this.local_sort_asc;
                                this.local_sort_col = SortColumn::Modified;
                                sort_entries(
                                    &mut this.local_entries,
                                    this.local_sort_col,
                                    this.local_sort_asc,
                                );
                            } else if this.sessions.is_empty() {
                                this.remote_sort_asc = !this.remote_sort_asc;
                                this.remote_sort_col = SortColumn::Modified;
                                sort_entries(
                                    &mut this.right_entries,
                                    this.remote_sort_col,
                                    this.remote_sort_asc,
                                );
                            } else if let Some(s) = this.sessions.get_mut(this.active_session_idx) {
                                this.remote_sort_asc = !this.remote_sort_asc;
                                this.remote_sort_col = SortColumn::Modified;
                                sort_entries(
                                    &mut s.entries,
                                    this.remote_sort_col,
                                    this.remote_sort_asc,
                                );
                            }
                            cx.notify();
                        }),
                    )
                    .child(t.col_modified()),
            )
            .when(self.show_permissions, |el| {
                el.child(self.render_col_resize_handle(2, cx))
                    .child(
                        div()
                            .id((prefix, 3usize))
                            .cursor_pointer()
                            .w(px(self.col_permissions_width))
                            .flex_shrink_0()
                            .px_2()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    if this.active_modal != ActiveModal::None { return; }
                                    if is_local {
                                        this.local_sort_asc = !this.local_sort_asc;
                                        this.local_sort_col = SortColumn::Permissions;
                                        sort_entries(
                                            &mut this.local_entries,
                                            this.local_sort_col,
                                            this.local_sort_asc,
                                        );
                                    } else if this.sessions.is_empty() {
                                        this.remote_sort_asc = !this.remote_sort_asc;
                                        this.remote_sort_col = SortColumn::Permissions;
                                        sort_entries(
                                            &mut this.right_entries,
                                            this.remote_sort_col,
                                            this.remote_sort_asc,
                                        );
                                    } else if let Some(s) = this.sessions.get_mut(this.active_session_idx) {
                                        this.remote_sort_asc = !this.remote_sort_asc;
                                        this.remote_sort_col = SortColumn::Permissions;
                                        sort_entries(
                                            &mut s.entries,
                                            this.remote_sort_col,
                                            this.remote_sort_asc,
                                        );
                                    }
                                    cx.notify();
                                }),
                            )
                            .child(t.col_permissions()),
                    )
            })
    }

    fn render_local_row(&self, idx: usize, cx: &mut Context<Self>) -> gpui::AnyElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_focused = self.active_pane == ActivePane::Local;
        let Some(entry) = self.local_entries.get(idx) else {
            return div().into_any_element();
        };
        let is_selected = idx == self.local_selected;
        let is_shift_selected = entry.selected;
        let is_space_marked = entry.space_marked;
        let is_blue = is_focused && (entry.selected || (is_selected && entry.name == ".."));
        let entry_path = PathBuf::from(&entry.path);
        let is_dir = entry.is_dir;
        let is_dotdot = entry.name == "..";
        let is_dark = theme.is_dark();
        let (row_icon, default_color) =
            get_file_icon(&entry.name, entry.is_dir, entry.is_symlink);
        let icon_color = if is_blue {
            0xffffff
        } else {
            theme.adjust_file_icon_color(default_color, is_dir)
        };

        let drag_items = {
            let mut marked: Vec<FileEntry> = self
                .local_entries
                .iter()
                .filter(|e| e.is_any_selected() && e.name != "..")
                .cloned()
                .collect();
            if marked.is_empty() {
                if entry.name != ".." {
                    vec![entry.clone()]
                } else {
                    Vec::new()
                }
            } else {
                if !marked.iter().any(|e| e.path == entry.path) && entry.name != ".." {
                    marked.push(entry.clone());
                }
                marked
            }
        };
        let drag_payload = DraggedFileItem::new(ActivePane::Local, drag_items);

        let is_overlay_open = self.active_modal != ActiveModal::None || self.context_menu.is_some();

        div()
            .id(("local-row", idx))
            .cursor_pointer()
            .w_full()
            .when(!is_dotdot, |el| {
                let tooltip_data = FileTooltip::from_entry(
                    entry,
                    &t,
                    is_dark,
                    theme.adjust_file_icon_color(icon_color, is_dir),
                );
                el.when(!is_overlay_open, |el| {
                    el.tooltip(move |_, cx| {
                        cx.new(|_| tooltip_data.clone())
                        .into()
                    })
                })
                .when(!is_overlay_open, |el| {
                    el.on_drag(drag_payload, |info: &DraggedFileItem, pos, _, cx| {
                        cx.new(|_| info.clone().position(pos))
                    })
                })
            })
            .flex()
            .items_center()
            .h(px(25.0))
            .px_3()
            .bg(if is_blue {
                theme.row_bg_selected()
            } else if is_space_marked {
                theme.row_bg_space_marked()
            } else {
                theme.row_bg_normal()
            })
            .border_b_1()
            .border_color(theme.row_border())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(
                    move |this, event: &MouseDownEvent, _, cx| {
                        if this.active_modal != ActiveModal::None { return; }
                        this.active_pane = ActivePane::Local;
                        this.session_dropdown_open = false;
                        this.favorites_menu_pane = None;
                        this.context_menu = None;
                        this.favorites_menu_pane = None;
                        cx.stop_propagation();
                        let is_cmd = event.modifiers.platform
                            || event.modifiers.control
                            || event.modifiers.secondary();
                        if event.modifiers.shift {
                            this.select_range_local(idx);
                        } else if is_cmd {
                            if let Some(e) = this.local_entries.get_mut(idx) {
                                if e.name != ".." {
                                    e.selected = !e.selected;
                                }
                            }
                            this.local_selected = idx;
                        } else if event.click_count >= 2 {
                            if is_dotdot || is_dir {
                                this.navigate_local(
                                    entry_path.clone(),
                                );
                            } else if crate::archive::is_archive_file(&entry_path.to_string_lossy()) && entry_path.is_file() {
                                this.navigate_local(
                                    entry_path.clone(),
                                );
                            } else if let Some((archive_path, inner_path)) = crate::archive::split_archive_path(&entry_path) {
                                if let Ok(temp_file) = crate::archive::extract_single_file_to_temp(&archive_path, &inner_path) {
                                    cx.open_with_system(&temp_file);
                                }
                            } else {
                                cx.open_with_system(&entry_path);
                            }
                        } else {
                            let is_already_marked = this.local_entries.get(idx).map(|e| e.is_any_selected()).unwrap_or(false);
                            if is_already_marked {
                                this.local_selected = idx;
                            } else {
                                for (i, e) in this.local_entries.iter_mut().enumerate() {
                                    if i == idx && e.name != ".." {
                                        e.selected = true;
                                    } else {
                                        e.selected = false;
                                        e.space_marked = false;
                                    }
                                }
                                this.local_selected = idx;
                            }
                        }
                        cx.notify();
                    },
                ),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(
                    move |this, event: &MouseDownEvent, _, cx| {
                        if this.active_modal != ActiveModal::None { return; }
                        this.active_pane = ActivePane::Local;
                        this.local_selected = idx;
                        this.session_dropdown_open = false;
                        this.favorites_menu_pane = None;
                        this.context_menu =
                            Some(ContextMenuState {
                                pane: ActivePane::Local,
                                position: event.position,
                            });
                        cx.notify();
                    },
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pr_3()
                    .flex()
                    .items_center()
                    .gap_2()
                    .overflow_hidden()
                    .child(render_icon(row_icon, 13.5, icon_color))
                    .child(
                        div()
                            .text_xs()
                            .overflow_hidden()
                            .text_ellipsis()
                            .whitespace_nowrap()
                            .font_weight(
                                if is_dir || is_space_marked || is_shift_selected {
                                    gpui::FontWeight::MEDIUM
                                } else {
                                    gpui::FontWeight::NORMAL
                                },
                            )
                            .text_color(
                                if is_blue {
                                    theme.row_text_selected()
                                } else if is_space_marked || is_dir {
                                    theme.row_text_dir()
                                } else {
                                    theme.row_text_normal()
                                },
                            )
                            .child(entry.name.clone()),
                    ),
            )
            .child(div().w(px(7.0)).flex_shrink_0().h_full())
            .child(
                div()
                    .w(px(self.col_size_width))
                    .flex_shrink_0()
                    .px_2()
                    .text_right()
                    .text_xs()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(if is_blue {
                        rgb(0xffffff)
                    } else if is_space_marked && entry.is_dir && entry.calculated_dir_size.is_some() {
                        theme.row_text_dir()
                    } else {
                        theme.row_text_secondary()
                    })
                    .child(
                        entry.formatted_size(t.folder_type_label()),
                    ),
            )
            .child(div().w(px(7.0)).flex_shrink_0().h_full())
            .child(
                div()
                    .w(px(self.col_modified_width))
                    .flex_shrink_0()
                    .px_2()
                    .text_xs()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(if is_blue {
                        rgb(0xffffff)
                    } else {
                        theme.row_text_secondary()
                    })
                    .child(entry.formatted_modified(self.date_format).to_string()),
            )
            .when(self.show_permissions, |el| {
                el.child(div().w(px(7.0)).flex_shrink_0().h_full())
                    .child(
                        div()
                            .w(px(self.col_permissions_width))
                            .flex_shrink_0()
                            .px_2()
                            .text_xs()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_color(if is_blue {
                                rgb(0xffffff)
                            } else {
                                theme.row_text_secondary()
                            })
                            .child(entry.permissions_str.clone()),
                    )
            })
            .into_any_element()
    }

    fn render_right_local_row(&self, idx: usize, cx: &mut Context<Self>) -> gpui::AnyElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_focused = self.active_pane == ActivePane::Remote;
        let Some(entry) = self.right_entries.get(idx) else {
            return div().into_any_element();
        };
        let is_selected = idx == self.right_selected;
        let is_shift_selected = entry.selected;
        let is_space_marked = entry.space_marked;
        let is_blue = is_focused && (entry.selected || (is_selected && entry.name == ".."));
        let entry_path = PathBuf::from(&entry.path);
        let is_dir = entry.is_dir;
        let is_dotdot = entry.name == "..";
        let is_dark = theme.is_dark();
        let (row_icon, default_color) =
            get_file_icon(&entry.name, entry.is_dir, entry.is_symlink);
        let icon_color = if is_blue {
            0xffffff
        } else {
            theme.adjust_file_icon_color(default_color, is_dir)
        };

        let drag_items = {
            let mut marked: Vec<FileEntry> = self
                .right_entries
                .iter()
                .filter(|e| e.is_any_selected() && e.name != "..")
                .cloned()
                .collect();
            if marked.is_empty() {
                if entry.name != ".." {
                    vec![entry.clone()]
                } else {
                    Vec::new()
                }
            } else {
                if !marked.iter().any(|e| e.path == entry.path) && entry.name != ".." {
                    marked.push(entry.clone());
                }
                marked
            }
        };
        let drag_payload = DraggedFileItem::new(ActivePane::Remote, drag_items);

        let is_overlay_open = self.active_modal != ActiveModal::None || self.context_menu.is_some();

        div()
            .id(("right-local-row", idx))
            .cursor_pointer()
            .w_full()
            .when(!is_dotdot, |el| {
                let tooltip_data = FileTooltip::from_entry(
                    entry,
                    &t,
                    is_dark,
                    theme.adjust_file_icon_color(icon_color, is_dir),
                );
                el.when(!is_overlay_open, |el| {
                    el.tooltip(move |_, cx| {
                        cx.new(|_| tooltip_data.clone())
                        .into()
                    })
                })
                .when(!is_overlay_open, |el| {
                    el.on_drag(drag_payload, |info: &DraggedFileItem, pos, _, cx| {
                        cx.new(|_| info.clone().position(pos))
                    })
                })
            })
            .flex()
            .items_center()
            .h(px(25.0))
            .px_3()
            .bg(if is_blue {
                theme.row_bg_selected()
            } else if is_space_marked {
                theme.row_bg_space_marked()
            } else {
                theme.row_bg_normal()
            })
            .border_b_1()
            .border_color(theme.row_border())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(
                    move |this, event: &MouseDownEvent, _, cx| {
                        if this.active_modal != ActiveModal::None { return; }
                        this.active_pane = ActivePane::Remote;
                        this.session_dropdown_open = false;
                        this.favorites_menu_pane = None;
                        this.context_menu = None;
                        cx.stop_propagation();
                        let is_cmd = event.modifiers.platform
                            || event.modifiers.control
                            || event.modifiers.secondary();
                        if event.modifiers.shift {
                            this.select_range_remote(idx);
                        } else if is_cmd {
                            if let Some(e) = this.right_entries.get_mut(idx) {
                                if e.name != ".." {
                                    e.selected = !e.selected;
                                }
                            }
                            this.right_selected = idx;
                        } else if event.click_count >= 2 {
                            if is_dotdot || is_dir {
                                this.navigate_right_local(
                                    entry_path.clone(),
                                );
                            } else if crate::archive::is_archive_file(&entry_path.to_string_lossy()) && entry_path.is_file() {
                                this.navigate_right_local(
                                    entry_path.clone(),
                                );
                            } else if let Some((archive_path, inner_path)) = crate::archive::split_archive_path(&entry_path) {
                                if let Ok(temp_file) = crate::archive::extract_single_file_to_temp(&archive_path, &inner_path) {
                                    cx.open_with_system(&temp_file);
                                }
                            } else {
                                cx.open_with_system(&entry_path);
                            }
                        } else {
                            let is_already_marked = this.right_entries.get(idx).map(|e| e.is_any_selected()).unwrap_or(false);
                            if is_already_marked {
                                this.right_selected = idx;
                            } else {
                                for (i, e) in this.right_entries.iter_mut().enumerate() {
                                    if i == idx && e.name != ".." {
                                        e.selected = true;
                                    } else {
                                        e.selected = false;
                                        e.space_marked = false;
                                    }
                                }
                                this.right_selected = idx;
                            }
                        }
                        cx.notify();
                    },
                ),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(
                    move |this, event: &MouseDownEvent, _, cx| {
                        if this.active_modal != ActiveModal::None { return; }
                        this.active_pane = ActivePane::Remote;
                        this.right_selected = idx;
                        this.session_dropdown_open = false;
                        this.favorites_menu_pane = None;
                        this.context_menu =
                            Some(ContextMenuState {
                                pane: ActivePane::Remote,
                                position: event.position,
                            });
                        cx.notify();
                    },
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pr_3()
                    .flex()
                    .items_center()
                    .gap_2()
                    .overflow_hidden()
                    .child(render_icon(row_icon, 13.5, icon_color))
                    .child(
                        div()
                            .text_xs()
                            .overflow_hidden()
                            .text_ellipsis()
                            .whitespace_nowrap()
                            .font_weight(
                                if is_dir || is_space_marked || is_shift_selected {
                                    gpui::FontWeight::MEDIUM
                                } else {
                                    gpui::FontWeight::NORMAL
                                },
                            )
                            .text_color(
                                if is_blue {
                                    theme.row_text_selected()
                                } else if is_space_marked || is_dir {
                                    theme.row_text_dir()
                                } else {
                                    theme.row_text_normal()
                                },
                            )
                            .child(entry.name.clone()),
                    ),
            )
            .child(div().w(px(7.0)).flex_shrink_0().h_full())
            .child(
                div()
                    .w(px(self.col_size_width))
                    .flex_shrink_0()
                    .px_2()
                    .text_right()
                    .text_xs()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(if is_blue {
                        rgb(0xffffff)
                    } else if is_space_marked && entry.is_dir && entry.calculated_dir_size.is_some() {
                        theme.row_text_dir()
                    } else {
                        theme.row_text_secondary()
                    })
                    .child(
                        entry.formatted_size(t.folder_type_label()),
                    ),
            )
            .child(div().w(px(7.0)).flex_shrink_0().h_full())
            .child(
                div()
                    .w(px(self.col_modified_width))
                    .flex_shrink_0()
                    .px_2()
                    .text_xs()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(if is_blue {
                        rgb(0xffffff)
                    } else {
                        theme.row_text_secondary()
                    })
                    .child(entry.formatted_modified(self.date_format).to_string()),
            )
            .when(self.show_permissions, |el| {
                el.child(div().w(px(7.0)).flex_shrink_0().h_full())
                    .child(
                        div()
                            .w(px(self.col_permissions_width))
                            .flex_shrink_0()
                            .px_2()
                            .text_xs()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_color(if is_blue {
                                rgb(0xffffff)
                            } else {
                                theme.row_text_secondary()
                            })
                            .child(entry.permissions_str.clone()),
                    )
            })
            .into_any_element()
    }

    fn render_remote_row(&self, idx: usize, cx: &mut Context<Self>) -> gpui::AnyElement {
        let t = self.i18n();
        let theme = self.theme();
        let is_focused = self.active_pane == ActivePane::Remote;
        let Some(session) = self.sessions.get(self.active_session_idx) else {
            return div().into_any_element();
        };
        let Some(entry) = session.entries.get(idx) else {
            return div().into_any_element();
        };
        let is_selected = idx == session.selected_index;
        let is_shift_selected = entry.selected;
        let is_space_marked = entry.space_marked;
        let env_color = session.env_color;
        let is_primary_selected = is_focused && (entry.selected || (is_selected && entry.name == ".."));
        let is_unfocused_selected = !is_focused && (entry.selected || (is_selected && entry.name == ".."));
        let is_dir = entry.is_dir || entry.is_symlink;
        let is_dotdot = entry.name == "..";
        let is_dark = theme.is_dark();
        let target_path = entry.path.clone();
        let (row_icon, default_color) =
            get_file_icon(&entry.name, entry.is_dir, entry.is_symlink);
        let icon_color = if is_primary_selected {
            0xffffff
        } else {
            theme.adjust_file_icon_color(default_color, is_dir)
        };

        let drag_items = {
            let mut marked: Vec<FileEntry> = session
                .entries
                .iter()
                .filter(|e| e.is_any_selected() && e.name != "..")
                .cloned()
                .collect();
            if marked.is_empty() {
                if entry.name != ".." {
                    vec![entry.clone()]
                } else {
                    Vec::new()
                }
            } else {
                if !marked.iter().any(|e| e.path == entry.path) && entry.name != ".." {
                    marked.push(entry.clone());
                }
                marked
            }
        };
        let drag_payload = DraggedFileItem::new(ActivePane::Remote, drag_items);

        let is_overlay_open = self.active_modal != ActiveModal::None || self.context_menu.is_some();

        div()
            .id(("remote-row", idx))
            .cursor_pointer()
            .w_full()
            .when(!is_dotdot, |el| {
                let tooltip_data = FileTooltip::from_entry(
                    entry,
                    &t,
                    is_dark,
                    theme.adjust_file_icon_color(icon_color, is_dir),
                );
                el.when(!is_overlay_open, |el| {
                    el.tooltip(move |_, cx| {
                        cx.new(|_| tooltip_data.clone())
                        .into()
                    })
                })
                .when(!is_overlay_open, |el| {
                    el.on_drag(drag_payload, |info: &DraggedFileItem, pos, _, cx| {
                        cx.new(|_| info.clone().position(pos))
                    })
                })
            })
            .flex()
            .items_center()
            .h(px(25.0))
            .px_3()
            .bg(if is_primary_selected {
                rgb(env_color)
            } else if is_space_marked {
                rgba(((env_color & 0xffffff) << 8) | 0x33)
            } else if is_unfocused_selected {
                rgba(((env_color & 0xffffff) << 8) | 0x24)
            } else {
                theme.row_bg_normal()
            })
            .border_b_1()
            .border_color(theme.row_border())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(
                    move |this, event: &MouseDownEvent, _, cx| {
                        if this.active_modal != ActiveModal::None { return; }
                        this.active_pane = ActivePane::Remote;
                        this.session_dropdown_open = false;
                        this.favorites_menu_pane = None;
                        this.context_menu = None;
                        cx.stop_propagation();

                        let is_cmd = event.modifiers.platform
                            || event.modifiers.control
                            || event.modifiers.secondary();
                        if event.modifiers.shift {
                            this.select_range_remote(idx);
                        } else if is_cmd {
                            if let Some(s) = this
                                .sessions
                                .get_mut(this.active_session_idx)
                            {
                                if let Some(e) =
                                    s.entries.get_mut(idx)
                                {
                                    if e.name != ".." {
                                        e.selected = !e.selected;
                                    }
                                }
                                s.selected_index = idx;
                            }
                        } else if event.click_count >= 2 {
                            if is_dotdot || is_dir {
                                this.navigate_remote(
                                    target_path.clone(),
                                );
                            } else {
                                if let Some(s) = this.sessions.get(this.active_session_idx) {
                                    if let Some(entry) = s.entries.get(idx).cloned() {
                                        this.open_remote_file_for_editing(entry, None, cx);
                                    }
                                }
                            }
                        } else {
                            if let Some(s) = this
                                .sessions
                                .get_mut(this.active_session_idx)
                            {
                                let is_already_marked = s.entries.get(idx).map(|e| e.is_any_selected()).unwrap_or(false);
                                if is_already_marked {
                                    s.selected_index = idx;
                                } else {
                                    for (i, e) in s.entries.iter_mut().enumerate() {
                                        if i == idx && e.name != ".." {
                                            e.selected = true;
                                        } else {
                                            e.selected = false;
                                            e.space_marked = false;
                                        }
                                    }
                                    s.selected_index = idx;
                                }
                            }
                        }
                        cx.notify();
                    },
                ),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(
                    move |this, event: &MouseDownEvent, _, cx| {
                        if this.active_modal != ActiveModal::None { return; }
                        this.active_pane = ActivePane::Remote;
                        if let Some(s) = this
                            .sessions
                            .get_mut(this.active_session_idx)
                        {
                            s.selected_index = idx;
                        }
                        this.session_dropdown_open = false;
                        this.favorites_menu_pane = None;
                        this.context_menu =
                            Some(ContextMenuState {
                                pane: ActivePane::Remote,
                                position: event.position,
                            });
                        cx.notify();
                    },
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pr_3()
                    .flex()
                    .items_center()
                    .gap_2()
                    .overflow_hidden()
                    .child(render_icon(row_icon, 13.5, icon_color))
                    .child(
                        div()
                            .text_xs()
                            .overflow_hidden()
                            .text_ellipsis()
                            .whitespace_nowrap()
                            .font_weight(
                                if is_dir || is_space_marked || is_shift_selected {
                                    gpui::FontWeight::MEDIUM
                                } else {
                                    gpui::FontWeight::NORMAL
                                },
                            )
                            .text_color(
                                if is_primary_selected {
                                    theme.row_text_selected()
                                } else if entry.is_symlink {
                                    if theme.is_dark() { rgb(0x2dd4bf) } else { rgb(0x0d9488) }
                                } else if is_dir || is_space_marked || is_unfocused_selected {
                                    theme.row_text_dir()
                                } else {
                                    theme.row_text_normal()
                                },
                            )
                            .child(entry.name.clone()),
                    ),
            )
            .child(div().w(px(7.0)).flex_shrink_0().h_full())
            .child(
                div()
                    .w(px(self.col_size_width))
                    .flex_shrink_0()
                    .px_2()
                    .text_right()
                    .text_xs()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(if is_primary_selected {
                        rgb(0xffffff)
                    } else if is_space_marked && entry.is_dir && entry.calculated_dir_size.is_some() {
                        theme.row_text_dir()
                    } else {
                        theme.row_text_secondary()
                    })
                    .child(
                        entry.formatted_size(t.folder_type_label()),
                    ),
            )
            .child(div().w(px(7.0)).flex_shrink_0().h_full())
            .child(
                div()
                    .w(px(self.col_modified_width))
                    .flex_shrink_0()
                    .px_2()
                    .text_xs()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(if is_primary_selected {
                        rgb(0xffffff)
                    } else {
                        theme.row_text_secondary()
                    })
                    .child(entry.formatted_modified(self.date_format).to_string()),
            )
            .when(self.show_permissions, |el| {
                el.child(div().w(px(7.0)).flex_shrink_0().h_full())
                    .child(
                        div()
                            .w(px(self.col_permissions_width))
                            .flex_shrink_0()
                            .px_2()
                            .text_xs()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_color(if is_primary_selected {
                                rgb(0xffffff)
                            } else {
                                theme.row_text_secondary()
                            })
                            .child(entry.permissions_str.clone()),
                    )
            })
            .into_any_element()
    }

}
