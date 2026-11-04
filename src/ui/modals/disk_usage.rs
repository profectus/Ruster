use gpui::{div, prelude::*, px, rgb, rgba, Context, IntoElement, MouseButton};
use crate::app::RusterView;
use crate::disk_usage::DiskUsageTab;
use crate::icons::render_icon;
use crate::model::{format_bytes, ActiveModal};
use std::path::{Path, PathBuf};

impl RusterView {
    pub fn render_disk_usage_modal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();

        let total_size_str = format_bytes(self.disk_usage_total_size);
        let selected_size_str = format_bytes(self.disk_usage_selected_size);
        let selected_path_str = self
            .disk_usage_selected_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| self.disk_usage_root.to_string_lossy().to_string());

        let total_bytes = self.disk_usage_total_size.max(1) as f32;
        let cats = &self.disk_usage_categories;
        let pct_build = ((cats.build_bytes as f32 / total_bytes) * 100.0).clamp(0.0, 100.0);
        let pct_deps = ((cats.deps_bytes as f32 / total_bytes) * 100.0).clamp(0.0, 100.0);
        let pct_archives = ((cats.archives_bytes as f32 / total_bytes) * 100.0).clamp(0.0, 100.0);
        let pct_git = ((cats.git_bytes as f32 / total_bytes) * 100.0).clamp(0.0, 100.0);
        let pct_media = ((cats.media_bytes as f32 / total_bytes) * 100.0).clamp(0.0, 100.0);
        let pct_other = (100.0 - (pct_build + pct_deps + pct_archives + pct_git + pct_media)).max(0.0);

        let breadcrumbs = get_path_breadcrumbs(&self.disk_usage_root);

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
                    .w(px(980.0))
                    .h(px(670.0))
                    .rounded(px(10.0))
                    .bg(theme.modal_bg())
                    .border_1()
                    .border_color(theme.border_strong())
                    .shadow_xl()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    .on_mouse_down(MouseButton::Right, cx.listener(|_, _, _, cx| cx.stop_propagation()))
                    // 1. Header
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_4()
                            .py_2p5()
                            .rounded_t(px(10.0))
                            .border_b_1()
                            .border_color(theme.border())
                            .flex()
                            .items_center()
                            .justify_between()
                            .bg(theme.modal_header_bg())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(render_icon("icons/hard-drive.svg", 16.0, theme.c_icon_active()))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .child(format!(
                                                "{}: {}",
                                                t.modal_disk_usage_title(),
                                                self.disk_usage_root.to_string_lossy()
                                            )),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .id("disk-usage-rescan")
                                            .cursor_pointer()
                                            .px_2p5()
                                            .py_1()
                                            .rounded(px(4.0))
                                            .bg(theme.button_secondary_bg())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .border_1()
                                            .border_color(theme.border())
                                            .flex()
                                            .items_center()
                                            .gap_1p5()
                                            .text_xs()
                                            .text_color(theme.text_primary())
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.start_disk_usage_scan(cx);
                                            }))
                                            .child(render_icon("icons/sync.svg", 12.0, theme.c_icon_secondary()))
                                            .child(t.btn_rescan()),
                                    )
                                    .child(
                                        div()
                                            .id("disk-usage-close-x")
                                            .cursor_pointer()
                                            .p_1()
                                            .rounded(px(4.0))
                                            .hover(|s| s.bg(theme.popover_item_active()))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.stop_disk_usage_scan();
                                                this.active_modal = ActiveModal::None;
                                                cx.notify();
                                            }))
                                            .child(render_icon("icons/close.svg", 14.0, theme.c_icon_secondary())),
                                    ),
                            ),
                    )
                    // 2. Clickable Path Breadcrumbs & Storage Switcher
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_4()
                            .py_2()
                            .border_b_1()
                            .border_color(theme.border())
                            .bg(theme.modal_card_bg())
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                // Breadcrumbs trail
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .text_xs()
                                    .overflow_hidden()
                                    .children(breadcrumbs.iter().enumerate().map(|(b_idx, (b_name, b_path))| {
                                        let is_last = b_idx == breadcrumbs.len().saturating_sub(1);
                                        let target_path = b_path.clone();

                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                if is_last {
                                                    div()
                                                        .id(("disk-usage-breadcrumb-last", b_idx))
                                                        .px_1p5()
                                                        .py_0p5()
                                                        .rounded(px(3.0))
                                                        .font_weight(gpui::FontWeight::BOLD)
                                                        .text_color(if theme.is_dark() { rgb(0xffffff) } else { rgb(0x0f172a) })
                                                        .child(b_name.clone())
                                                } else {
                                                    div()
                                                        .id(("disk-usage-breadcrumb", b_idx))
                                                        .cursor_pointer()
                                                        .px_1p5()
                                                        .py_0p5()
                                                        .rounded(px(3.0))
                                                        .text_color(if theme.is_dark() { rgb(0x60a5fa) } else { rgb(0x2563eb) })
                                                        .hover(|s| s.bg(theme.button_hover_bg()).underline())
                                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                            this.disk_usage_root = target_path.clone();
                                                            this.start_disk_usage_scan(cx);
                                                        }))
                                                        .child(b_name.clone())
                                                }
                                            )
                                            .when(!is_last, |el| {
                                                el.child(
                                                    div()
                                                        .text_color(theme.text_muted())
                                                        .child("›")
                                                )
                                            })
                                    })),
                            )
                            // Quick Destination & Storage Switcher Buttons
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .flex_shrink_0()
                                    // Home directory shortcut (icon only)
                                    .child(
                                        div()
                                            .id("disk-usage-home-btn")
                                            .cursor_pointer()
                                            .w(px(24.0))
                                            .h(px(22.0))
                                            .rounded(px(3.0))
                                            .bg(theme.button_secondary_bg())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .border_1()
                                            .border_color(theme.border())
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                if let Ok(home) = std::env::var("HOME") {
                                                    this.disk_usage_root = PathBuf::from(home);
                                                    this.start_disk_usage_scan(cx);
                                                }
                                            }))
                                            .child(render_icon("icons/home.svg", 13.0, theme.c_icon_secondary())),
                                    )
                                    // Volumes / Mounted disks shortcut (icon only)
                                    .child(
                                        div()
                                            .id("disk-usage-volumes-btn")
                                            .cursor_pointer()
                                            .w(px(24.0))
                                            .h(px(22.0))
                                            .rounded(px(3.0))
                                            .bg(theme.button_secondary_bg())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .border_1()
                                            .border_color(theme.border())
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                let volumes = PathBuf::from("/Volumes");
                                                if volumes.exists() {
                                                    this.disk_usage_root = volumes;
                                                } else {
                                                    this.disk_usage_root = PathBuf::from("/");
                                                }
                                                this.start_disk_usage_scan(cx);
                                            }))
                                            .child(render_icon("icons/hard-drive.svg", 13.0, theme.c_icon_secondary())),
                                    ),
                            ),
                    )
                    // 3. Summary & Category Bar
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_4()
                            .py_2()
                            .border_b_1()
                            .border_color(theme.border())
                            .bg(theme.modal_card_bg())
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            // Total counts
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .text_xs()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2p5()
                                            .text_color(theme.text_muted())
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(t.lbl_total())
                                                    .child(
                                                        div()
                                                            .font_weight(gpui::FontWeight::BOLD)
                                                            .text_color(theme.text_primary())
                                                            .child(total_size_str),
                                                    ),
                                            )
                                            .child(div().text_color(theme.border_strong()).child("•"))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(t.lbl_files())
                                                    .child(
                                                        div()
                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                            .text_color(theme.text_primary())
                                                            .child(self.disk_usage_total_files.to_string()),
                                                    ),
                                            )
                                            .child(div().text_color(theme.border_strong()).child("•"))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(t.lbl_folders())
                                                    .child(
                                                        div()
                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                            .text_color(theme.text_primary())
                                                            .child(self.disk_usage_total_dirs.to_string()),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        if self.disk_usage_scanning {
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .w(px(7.0))
                                                        .h(px(7.0))
                                                        .rounded_full()
                                                        .bg(rgb(0x10b981)),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x10b981))
                                                        .font_weight(gpui::FontWeight::MEDIUM)
                                                        .child(t.lbl_scanning_files_count(
                                                            self.disk_usage_scanned_files_live,
                                                            &format_bytes(self.disk_usage_scanned_bytes_live),
                                                        )),
                                                )
                                        } else {
                                            div().text_xs().text_color(theme.text_muted()).child(t.lbl_scan_completed())
                                        }
                                    ),
                            )
                            // Segmented Category Bar
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .w_full()
                                            .h(px(10.0))
                                            .rounded_full()
                                            .overflow_hidden()
                                            .bg(theme.progress_bg())
                                            .border_1()
                                            .border_color(theme.progress_border())
                                            .flex()
                                            .p(px(0.5))
                                            .gap(px(1.0))
                                            // Build (blue)
                                            .when(pct_build > 0.5, |el| {
                                                el.child(
                                                    div()
                                                        .h_full()
                                                        .w(gpui::relative(pct_build / 100.0))
                                                        .bg(rgb(0x3b82f6))
                                                        .rounded_l_full(),
                                                )
                                            })
                                            // Deps (amber)
                                            .when(pct_deps > 0.5, |el| {
                                                el.child(
                                                    div()
                                                        .h_full()
                                                        .w(gpui::relative(pct_deps / 100.0))
                                                        .bg(rgb(0xf59e0b)),
                                                )
                                            })
                                            // Archives (emerald)
                                            .when(pct_archives > 0.5, |el| {
                                                el.child(
                                                    div()
                                                        .h_full()
                                                        .w(gpui::relative(pct_archives / 100.0))
                                                        .bg(rgb(0x10b981)),
                                                )
                                            })
                                            // Git (purple)
                                            .when(pct_git > 0.5, |el| {
                                                el.child(
                                                    div()
                                                        .h_full()
                                                        .w(gpui::relative(pct_git / 100.0))
                                                        .bg(rgb(0x8b5cf6)),
                                                )
                                            })
                                            // Media (rose)
                                            .when(pct_media > 0.5, |el| {
                                                el.child(
                                                    div()
                                                        .h_full()
                                                        .w(gpui::relative(pct_media / 100.0))
                                                        .bg(rgb(0xf43f5e)),
                                                )
                                            })
                                            // Other (slate)
                                            .when(pct_other > 0.5, |el| {
                                                el.child(
                                                    div()
                                                        .h_full()
                                                        .w(gpui::relative(pct_other / 100.0))
                                                        .bg(rgb(0x64748b))
                                                        .rounded_r_full(),
                                                )
                                            }),
                                    )
                                    // Category Badges Row
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_3()
                                            .text_xs()
                                            .text_color(theme.text_secondary())
                                            .child(self.render_cat_pill(0x3b82f6, &format!("{} ({:.1}%)", t.cat_build(), pct_build)))
                                            .child(self.render_cat_pill(0xf59e0b, &format!("{} ({:.1}%)", t.cat_dependencies(), pct_deps)))
                                            .child(self.render_cat_pill(0x10b981, &format!("{} ({:.1}%)", t.cat_archives(), pct_archives)))
                                            .child(self.render_cat_pill(0x8b5cf6, &format!("{} ({:.1}%)", t.cat_git(), pct_git)))
                                            .child(self.render_cat_pill(0xf43f5e, &format!("{} ({:.1}%)", t.cat_media(), pct_media)))
                                            .child(self.render_cat_pill(0x64748b, &format!("{} ({:.1}%)", t.cat_other(), pct_other))),
                                    ),
                            ),
                    )
                    // 4. Tabs & Toolbar
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_4()
                            .bg(theme.modal_header_bg())
                            .border_b_1()
                            .border_color(theme.border())
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(self.render_tab_btn(
                                        DiskUsageTab::Tree,
                                        t.tab_directory_tree(),
                                        "icons/folder-tree.svg",
                                        cx,
                                    ))
                                    .child(self.render_tab_btn(
                                        DiskUsageTab::Extensions,
                                        t.tab_file_extensions(),
                                        "icons/file-code.svg",
                                        cx,
                                    ))
                                    .child(self.render_tab_btn(
                                        DiskUsageTab::TopFiles,
                                        t.tab_top_files(),
                                        "icons/star.svg",
                                        cx,
                                    )),
                            )
                            // Tree controls (Expand / Collapse all)
                            .when(self.disk_usage_tab == DiskUsageTab::Tree, |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .id("disk-usage-expand-all")
                                                .cursor_pointer()
                                                .px_2()
                                                .py_1()
                                                .rounded(px(4.0))
                                                .bg(theme.button_secondary_bg())
                                                .hover(|s| s.bg(theme.button_hover_bg()))
                                                .border_1()
                                                .border_color(theme.border())
                                                .text_xs()
                                                .text_color(theme.text_secondary())
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                    this.expand_all_disk_usage();
                                                    cx.notify();
                                                }))
                                                .child(format!("⊞ {}", t.btn_expand_all())),
                                        )
                                        .child(
                                            div()
                                                .id("disk-usage-collapse-all")
                                                .cursor_pointer()
                                                .px_2()
                                                .py_1()
                                                .rounded(px(4.0))
                                                .bg(theme.button_secondary_bg())
                                                .hover(|s| s.bg(theme.button_hover_bg()))
                                                .border_1()
                                                .border_color(theme.border())
                                                .text_xs()
                                                .text_color(theme.text_secondary())
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                    this.collapse_all_disk_usage();
                                                    cx.notify();
                                                }))
                                                .child(format!("⊟ {}", t.btn_collapse_all())),
                                        ),
                                )
                            }),
                    )
                    // 5. Main Tab Content (with rich loading state when scanning)
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(match self.disk_usage_tab {
                                DiskUsageTab::Tree => self.render_tree_view(cx).into_any_element(),
                                DiskUsageTab::Extensions => self.render_extensions_view(cx).into_any_element(),
                                DiskUsageTab::TopFiles => self.render_top_files_view(cx).into_any_element(),
                            }),
                    )
                    // 6. Bottom Action Bar
                    .child(
                        div()
                            .flex_shrink_0()
                            .px_4()
                            .py_2p5()
                            .rounded_b(px(10.0))
                            .border_t_1()
                            .border_color(theme.border())
                            .bg(theme.modal_header_bg())
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.text_secondary())
                                            .child(t.lbl_selected()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_primary())
                                            .overflow_hidden()
                                            .child(selected_path_str),
                                    )
                                    .child(
                                        div()
                                            .px_1p5()
                                            .py_0p5()
                                            .rounded(px(3.0))
                                            .bg(theme.popover_item_active())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgb(0x3b82f6))
                                            .child(selected_size_str),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    // Reveal in Finder
                                    .child(
                                        div()
                                            .id("disk-usage-reveal-finder")
                                            .cursor_pointer()
                                            .px_2p5()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.button_secondary_bg())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_primary())
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                if let Some(path) = &this.disk_usage_selected_path {
                                                    cx.reveal_path(path);
                                                } else {
                                                    cx.reveal_path(&this.disk_usage_root);
                                                }
                                            }))
                                            .child(t.act_reveal_in_finder()),
                                    )
                                    // Delete / Move to Trash
                                    .child(
                                        div()
                                            .id("disk-usage-delete-trash")
                                            .cursor_pointer()
                                            .px_2p5()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(rgba(0xef444415))
                                            .hover(|s| s.bg(rgba(0xef444430)))
                                            .border_1()
                                            .border_color(rgba(0xef444440))
                                            .flex()
                                            .items_center()
                                            .gap_1p5()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(rgb(0xef4444))
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.delete_disk_usage_selected(cx);
                                            }))
                                            .child(render_icon("icons/trash.svg", 12.0, 0xef4444))
                                            .child(t.btn_move_to_trash()),
                                    )
                                    // Close Button
                                    .child(
                                        div()
                                            .id("disk-usage-close-btn")
                                            .cursor_pointer()
                                            .px_3()
                                            .py_1p5()
                                            .rounded(px(5.0))
                                            .bg(theme.button_secondary_bg())
                                            .hover(|s| s.bg(theme.button_hover_bg()))
                                            .border_1()
                                            .border_color(theme.border())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.text_primary())
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                                this.stop_disk_usage_scan();
                                                this.active_modal = ActiveModal::None;
                                                cx.notify();
                                            }))
                                            .child(t.btn_close()),
                                    ),
                            ),
                    ),
            )
    }

    fn render_cat_pill(&self, color: u32, label: &str) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1p5()
            .child(
                div()
                    .w(px(7.0))
                    .h(px(7.0))
                    .rounded_full()
                    .bg(rgb(color)),
            )
            .child(label.to_string())
    }

    fn render_tab_btn(
        &self,
        tab: DiskUsageTab,
        label: &'static str,
        icon: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.disk_usage_tab == tab;
        let theme = self.theme();

        div()
            .id(("disk-usage-tab", match tab {
                DiskUsageTab::Tree => 1usize,
                DiskUsageTab::Extensions => 2usize,
                DiskUsageTab::TopFiles => 3usize,
            }))
            .cursor_pointer()
            .px_3()
            .py_2()
            .border_b_2()
            .border_color(if is_active { rgb(0x2563eb) } else { rgba(0x00000000) })
            .flex()
            .items_center()
            .gap_1p5()
            .text_xs()
            .font_weight(if is_active { gpui::FontWeight::SEMIBOLD } else { gpui::FontWeight::NORMAL })
            .text_color(if is_active {
                if theme.is_dark() { rgb(0xffffff) } else { rgb(0x2563eb) }
            } else {
                theme.text_secondary()
            })
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                this.disk_usage_tab = tab;
                cx.notify();
            }))
            .child(render_icon(icon, 13.0, if is_active { 0x2563eb } else { theme.c_icon_secondary() }))
            .child(label)
    }

    // --- TAB 1: TreeView ---
    fn render_tree_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let t = self.i18n();

        // 1. If currently scanning and no rows yet, show prominent loading state!
        if self.disk_usage_scanning && self.disk_usage_rows.is_empty() {
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .p_8()
                .child(
                    div()
                        .w(px(52.0))
                        .h(px(52.0))
                        .rounded_full()
                        .bg(rgba(0x2563eb18))
                        .border_2()
                        .border_color(rgb(0x2563eb))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(render_icon("icons/sync.svg", 26.0, 0x2563eb)),
                )
                .child(
                    div()
                        .text_base()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.text_primary())
                        .child(t.lbl_analyzing_disk_space()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted())
                        .max_w(px(640.0))
                        .overflow_hidden()
                        .child(format!("{} {}", t.lbl_current(), self.disk_usage_scanned_current_dir)),
                )
                // Live progress stats pill
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_4()
                        .py_2()
                        .rounded(px(6.0))
                        .bg(theme.modal_card_bg())
                        .border_1()
                        .border_color(theme.border())
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_secondary())
                                .child(t.lbl_scanned_folders_count(self.disk_usage_scanned_dirs_live)),
                        )
                        .child(div().text_color(theme.border_strong()).child("•"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_secondary())
                                .child(t.lbl_scanned_files_count(self.disk_usage_scanned_files_live)),
                        )
                        .child(div().text_color(theme.border_strong()).child("•"))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(0x2563eb))
                                .child(format_bytes(self.disk_usage_scanned_bytes_live)),
                        ),
                )
                .child(
                    div()
                        .id("disk-usage-stop-loading-btn")
                        .cursor_pointer()
                        .px_3p5()
                        .py_1p5()
                        .rounded(px(5.0))
                        .bg(theme.button_secondary_bg())
                        .hover(|s| s.bg(theme.button_hover_bg()))
                        .border_1()
                        .border_color(theme.border())
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.text_primary())
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.stop_disk_usage_scan();
                            cx.notify();
                        }))
                        .child(t.btn_stop()),
                );
        }

        // 2. Main Tree Table
        div()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            // Table Header
            .child(
                div()
                    .flex_shrink_0()
                    .px_3()
                    .py_1p5()
                    .bg(theme.table_header_bg())
                    .border_b_1()
                    .border_color(theme.border())
                    .flex()
                    .items_center()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.text_secondary())
                    .child(div().w(px(350.0)).child(t.col_folder_or_file()))
                    .child(div().w(px(180.0)).child(t.col_usage()))
                    .child(div().w(px(100.0)).text_right().child(t.col_size()))
                    .child(div().w(px(80.0)).text_right().child(t.col_share()))
                    .child(div().w(px(90.0)).text_right().child(t.col_files()))
                    .child(div().w(px(130.0)).text_right().child(t.col_modified())),
            )
            // Table Rows
            .child(
                div()
                    .id("disk-usage-tree-list")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_1()
                    .flex()
                    .flex_col()
                    .children(
                        self.disk_usage_rows.iter().enumerate().map(|(idx, row)| {
                            let is_selected = idx == self.disk_usage_selected_row_idx;
                            let is_even = idx % 2 == 0;
                            let row_bg = if is_selected {
                                theme.row_bg_selected()
                            } else if is_even {
                                theme.pane_bg()
                            } else {
                                theme.modal_card_bg()
                            };

                            // HIGH-CONTRAST HOVER COLOR:
                            // If selected: slightly deeper blue on hover so text stays white!
                            // If not selected: in dark mode 0x282832, in light mode 0xe2e8f0 (clearly visible)!
                            let hover_bg = if is_selected {
                                rgb(0x1d4ed8)
                            } else if theme.is_dark() {
                                rgb(0x282832)
                            } else {
                                rgb(0xe2e8f0)
                            };

                            let indent = (row.depth as f32 * 18.0) + 4.0;
                            let node_id = row.node_id;
                            let row_path = row.path.clone();
                            let row_size = row.size;
                            let row_is_dir = row.is_dir;

                            let icon = if row.is_dir {
                                "icons/folder.svg"
                            } else {
                                "icons/file.svg"
                            };

                            let chevron = if row.is_dir && row.has_children {
                                if row.is_expanded { "▼" } else { "▶" }
                            } else {
                                " "
                            };

                            let bar_pct = row.percentage_of_parent.clamp(1.5, 100.0);
                            let bar_color = if is_selected {
                                rgb(0xffffff)
                            } else if bar_pct > 70.0 {
                                rgb(0x2563eb)
                            } else if bar_pct > 35.0 {
                                rgb(0x0284c7)
                            } else {
                                rgb(0x64748b)
                            };

                            div()
                                .id(("disk-usage-row", idx))
                                .px_2()
                                .py_1()
                                .rounded(px(3.0))
                                .bg(row_bg)
                                .hover(move |s| s.bg(hover_bg))
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                    this.disk_usage_selected_row_idx = idx;
                                    this.disk_usage_selected_path = Some(row_path.clone());
                                    this.disk_usage_selected_size = row_size;
                                    this.disk_usage_selected_is_dir = row_is_dir;
                                    if event.click_count >= 2 {
                                        this.navigate_to_disk_usage_selected(cx);
                                        return;
                                    }
                                    cx.notify();
                                }))
                                .flex()
                                .items_center()
                                .text_xs()
                                // Name with indent and chevron
                                .child(
                                    div()
                                        .w(px(350.0))
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(div().w(px(indent)))
                                        // Clickable expand/collapse chevron
                                        .child(
                                            div()
                                                .w(px(14.0))
                                                .h(px(14.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .cursor_pointer()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(if is_selected { rgb(0xffffff) } else { theme.text_muted() })
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    this.toggle_disk_usage_row(node_id);
                                                    cx.notify();
                                                }))
                                                .child(chevron),
                                        )
                                        .child(render_icon(icon, 13.0, if is_selected { 0xffffff } else if row.is_dir { 0xf59e0b } else { theme.c_icon_secondary() }))
                                        .child(
                                            div()
                                                .overflow_hidden()
                                                .font_weight(if is_selected || row.is_dir { gpui::FontWeight::SEMIBOLD } else { gpui::FontWeight::NORMAL })
                                                .text_color(if is_selected { theme.row_text_selected() } else { theme.text_primary() })
                                                .child(row.name.clone()),
                                        ),
                                )
                                // Usage Mini-bar
                                .child(
                                    div()
                                        .w(px(180.0))
                                        .pr_4()
                                        .child(
                                            div()
                                                .w_full()
                                                .h(px(10.0))
                                                .rounded(px(2.0))
                                                .bg(if is_selected { rgba(0xffffff40) } else { theme.progress_bg() })
                                                .border_1()
                                                .border_color(if is_selected { rgba(0xffffff60) } else { theme.progress_border() })
                                                .p(px(0.5))
                                                .child(
                                                    div()
                                                        .h_full()
                                                        .w(gpui::relative(bar_pct / 100.0))
                                                        .rounded(px(1.5))
                                                        .bg(bar_color),
                                                ),
                                        ),
                                )
                                // Size
                                .child(
                                    div()
                                        .w(px(100.0))
                                        .text_right()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(if is_selected { theme.row_text_selected() } else { theme.text_primary() })
                                        .child(format_bytes(row.size)),
                                )
                                // Share (%)
                                .child(
                                    div()
                                        .w(px(80.0))
                                        .text_right()
                                        .text_color(if is_selected { theme.row_text_selected() } else { theme.text_muted() })
                                        .child(format!("{:.1} %", row.percentage_of_parent)),
                                )
                                // Files count
                                .child(
                                    div()
                                        .w(px(90.0))
                                        .text_right()
                                        .text_color(if is_selected { theme.row_text_selected() } else { theme.text_muted() })
                                        .child(row.files_count.to_string()),
                                )
                                // Modified date
                                .child(
                                    div()
                                        .w(px(130.0))
                                        .text_right()
                                        .text_color(if is_selected { theme.row_text_selected() } else { theme.text_muted() })
                                        .child(row.modified_str.clone()),
                                )
                        }),
                    ),
            )
    }

    // --- TAB 2: Extensions View ---
    fn render_extensions_view(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let t = self.i18n();
        let total_size = self.disk_usage_total_size.max(1) as f32;

        div()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            // Header
            .child(
                div()
                    .flex_shrink_0()
                    .px_4()
                    .py_1p5()
                    .bg(theme.table_header_bg())
                    .border_b_1()
                    .border_color(theme.border())
                    .flex()
                    .items_center()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.text_secondary())
                    .child(div().w(px(160.0)).child(t.col_extension()))
                    .child(div().w(px(240.0)).child(t.col_category_description()))
                    .child(div().w(px(200.0)).child(t.col_usage()))
                    .child(div().w(px(130.0)).text_right().child(t.col_total_size()))
                    .child(div().w(px(130.0)).text_right().child(t.col_file_count())),
            )
            // Rows
            .child(
                div()
                    .id("disk-usage-ext-list")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_1()
                    .flex()
                    .flex_col()
                    .children(
                        self.disk_usage_extensions.iter().enumerate().map(|(idx, ext)| {
                            let is_even = idx % 2 == 0;
                            let row_bg = if is_even { theme.pane_bg() } else { theme.modal_card_bg() };
                            let hover_bg = if theme.is_dark() { rgb(0x282832) } else { rgb(0xe2e8f0) };
                            let pct = ((ext.size as f32 / total_size) * 100.0).clamp(1.0, 100.0);

                            div()
                                .id(("disk-usage-ext", idx))
                                .px_3()
                                .py_1p5()
                                .rounded(px(3.0))
                                .bg(row_bg)
                                .hover(move |s| s.bg(hover_bg))
                                .flex()
                                .items_center()
                                .text_xs()
                                .child(
                                    div()
                                        .w(px(160.0))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(if theme.is_dark() { rgb(0x60a5fa) } else { rgb(0x2563eb) })
                                        .child(if ext.extension == "(no extension)" {
                                            t.ext_category_name("(no extension)").to_string()
                                        } else {
                                            ext.extension.clone()
                                        }),
                                )
                                .child(
                                    div()
                                        .w(px(240.0))
                                        .text_color(theme.text_primary())
                                        .child(t.ext_category_name(&ext.category)),
                                )
                                .child(
                                    div()
                                        .w(px(200.0))
                                        .pr_4()
                                        .child(
                                            div()
                                                .w_full()
                                                .h(px(10.0))
                                                .rounded(px(2.0))
                                                .bg(theme.progress_bg())
                                                .border_1()
                                                .border_color(theme.progress_border())
                                                .p(px(0.5))
                                                .child(
                                                    div()
                                                        .h_full()
                                                        .w(gpui::relative(pct / 100.0))
                                                        .rounded(px(1.5))
                                                        .bg(rgb(0x3b82f6)),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .w(px(130.0))
                                        .text_right()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme.text_primary())
                                        .child(format_bytes(ext.size)),
                                )
                                .child(
                                    div()
                                        .w(px(130.0))
                                        .text_right()
                                        .text_color(theme.text_muted())
                                        .child(ext.count.to_string()),
                                )
                        }),
                    ),
            )
    }

    // --- TAB 3: Top 100 Files View ---
    fn render_top_files_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let t = self.i18n();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            // Header
            .child(
                div()
                    .flex_shrink_0()
                    .px_4()
                    .py_1p5()
                    .bg(theme.table_header_bg())
                    .border_b_1()
                    .border_color(theme.border())
                    .flex()
                    .items_center()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.text_secondary())
                    .child(div().w(px(340.0)).child(t.col_file_name()))
                    .child(div().flex_1().child(t.col_path()))
                    .child(div().w(px(120.0)).text_right().child(t.col_size()))
                    .child(div().w(px(130.0)).text_right().child(t.col_date_modified())),
            )
            // Rows
            .child(
                div()
                    .id("disk-usage-top-list")
                    .flex_1()
                    .overflow_y_scroll()
                    .p_1()
                    .flex()
                    .flex_col()
                    .children(
                        self.disk_usage_top_files.iter().enumerate().map(|(idx, file)| {
                            let is_selected = self.disk_usage_selected_path.as_ref() == Some(&file.path);
                            let is_even = idx % 2 == 0;
                            let row_bg = if is_selected {
                                theme.row_bg_selected()
                            } else if is_even {
                                theme.pane_bg()
                            } else {
                                theme.modal_card_bg()
                            };

                            let hover_bg = if is_selected {
                                rgb(0x1d4ed8)
                            } else if theme.is_dark() {
                                rgb(0x282832)
                            } else {
                                rgb(0xe2e8f0)
                            };

                            let file_path = file.path.clone();
                            let file_size = file.size;

                            div()
                                .id(("disk-usage-top", idx))
                                .px_3()
                                .py_1p5()
                                .rounded(px(3.0))
                                .bg(row_bg)
                                .hover(move |s| s.bg(hover_bg))
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                                    this.disk_usage_selected_path = Some(file_path.clone());
                                    this.disk_usage_selected_size = file_size;
                                    this.disk_usage_selected_is_dir = false;
                                    if event.click_count >= 2 {
                                        this.navigate_to_disk_usage_selected(cx);
                                        return;
                                    }
                                    cx.notify();
                                }))
                                .flex()
                                .items_center()
                                .text_xs()
                                .child(
                                    div()
                                        .w(px(340.0))
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .child(render_icon("icons/file.svg", 13.0, if is_selected { 0xffffff } else { theme.c_icon_secondary() }))
                                        .child(
                                            div()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(if is_selected { theme.row_text_selected() } else { theme.text_primary() })
                                                .overflow_hidden()
                                                .child(file.name.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .overflow_hidden()
                                        .text_color(if is_selected { theme.row_text_selected() } else { theme.text_muted() })
                                        .child(file.path.to_string_lossy().to_string()),
                                )
                                .child(
                                    div()
                                        .w(px(120.0))
                                        .text_right()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(if is_selected { theme.row_text_selected() } else { rgb(0xf43f5e) })
                                        .child(format_bytes(file.size)),
                                )
                                .child(
                                    div()
                                        .w(px(130.0))
                                        .text_right()
                                        .text_color(if is_selected { theme.row_text_selected() } else { theme.text_muted() })
                                        .child(file.modified_str.clone()),
                                )
                        }),
                    ),
            )
    }
}

fn get_path_breadcrumbs(path: &Path) -> Vec<(String, PathBuf)> {
    let mut segments = Vec::new();
    let mut current = Some(path);
    while let Some(p) = current {
        let name = p.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                if p == Path::new("/") {
                    "/".to_string()
                } else {
                    p.to_string_lossy().to_string()
                }
            });
        if !name.is_empty() {
            segments.push((name, p.to_path_buf()));
        }
        current = p.parent();
    }
    segments.reverse();
    if segments.is_empty() {
        segments.push(("/".to_string(), PathBuf::from("/")));
    }
    segments
}
