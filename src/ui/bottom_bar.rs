use gpui::{div, prelude::*, px, rgb, rgba, Context, IntoElement, MouseButton, MouseDownEvent};
use crate::app::RusterView;
use crate::icons::{get_file_icon, render_icon};
use crate::model::{format_bytes, ServerLogKind, TransferDirection, TransferQueueItem, TransferStatus};
use crate::theme::Theme;

impl RusterView {
    pub fn render_bottom_function_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(26.0))
            .bg(theme.bottom_bar_bg())
            .border_t_1()
            .border_color(theme.border())
            .px_2()
            .gap_1()
            .child(self.render_fn_btn("fn-f3", "F3", t.act_open(), cx, |this, cx| {
                this.open_selected_item(cx);
                cx.notify();
            }))
            .child(self.render_fn_btn("fn-f4", "F4", t.act_edit(), cx, |this, cx| {
                this.open_selected_item(cx);
                cx.notify();
            }))
            .child(self.render_fn_btn("fn-f5", "F5", t.act_copy(), cx, |this, cx| {
                this.copy_selected_item(false, cx);
                cx.notify();
            }))
            .child(self.render_fn_btn("fn-f6", "F6", t.act_move(), cx, |this, cx| {
                this.copy_selected_item(true, cx);
                cx.notify();
            }))
            .child(self.render_fn_btn("fn-f7", "F7", t.act_new_folder(), cx, |this, cx| {
                this.open_new_folder_modal();
                cx.notify();
            }))
            .child(self.render_fn_btn("fn-f8", "F8", t.act_delete(), cx, |this, cx| {
                this.open_delete_modal();
                cx.notify();
            }))
    }

    pub fn render_fn_btn(
        &self,
        id: &'static str,
        fkey: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
        handler: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let theme = self.theme();
        let clean_label = label.trim_end_matches("...");
        div()
            .id(id)
            .cursor_pointer()
            .flex_1()
            .h(px(20.0))
            .flex()
            .items_center()
            .justify_center()
            .gap_1()
            .bg(theme.button_bg())
            .border_1()
            .border_color(theme.border())
            .rounded(px(3.0))
            .text_xs()
            .hover(|s| s.bg(theme.button_hover_bg()))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    handler(this, cx);
                }),
            )
            .child(
                div()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.text_accent())
                    .child(fkey),
            )
            .child(
                div()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.text_primary())
                    .child(clean_label.to_string()),
            )
    }

    pub fn render_queue_resize_handle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let is_resizing = self.resizing_queue_panel.is_some();
        div()
            .id("queue-panel-resizer")
            .cursor_row_resize()
            .h(px(5.0))
            .w_full()
            .mt(px(-2.5))
            .mb(px(-2.5))
            .bg(if is_resizing { theme.border_focused() } else { rgba(0x00000000) })
            .hover(|s| s.bg(theme.border_focused()))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    if event.click_count == 2 {
                        this.queue_panel_height = 140.0;
                        this.resizing_queue_panel = None;
                        this.persist_app_state();
                        cx.notify();
                        return;
                    }
                    this.resizing_queue_panel = Some(crate::app::ResizeQueuePanelState {
                        start_mouse_y: f32::from(event.position.y),
                        initial_height: this.queue_panel_height,
                    });
                    cx.notify();
                }),
            )
    }

    pub fn render_transfer_queue_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.i18n();
        let theme = self.theme();
        let show_log = self.show_transfer_log;

        div()
            .flex()
            .flex_col()
            .h(px(self.queue_panel_height))
            .bg(theme.bottom_panel_bg())
            .border_t_1()
            .border_color(theme.border())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(24.0))
                    .px_2()
                    .bg(theme.top_bar_bg())
                    .border_b_1()
                    .border_color(theme.border())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(render_icon("icons/transfer.svg", 12.0, theme.c_icon_active()))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.text_primary())
                                    .child(t.transfer_queue_title()),
                            )
                            .when(!self.transfer_queue.is_empty(), |el| {
                                let active_count = self.transfer_queue.iter().filter(|i| !i.is_dir && i.status == TransferStatus::Transferring).count();
                                let queued_count = self.transfer_queue.iter().filter(|i| !i.is_dir && i.status == TransferStatus::Queued).count();
                                let label = if active_count > 0 {
                                    format!("{} aktivní, {} ve frontě", active_count, queued_count)
                                } else {
                                    format!("{} ve frontě", queued_count)
                                };
                                el.child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(3.0))
                                        .bg(theme.button_secondary_bg())
                                        .text_xs()
                                        .text_color(theme.text_secondary())
                                        .child(label),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            // Cancel / clear transfer queue button
                            .when(!self.transfer_queue.is_empty(), |el| {
                                el.child(
                                    div()
                                        .id("btn-cancel-transfer-queue")
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .px_2()
                                        .h(px(18.0))
                                        .rounded(px(3.0))
                                        .bg(if theme.is_dark() { rgb(0x381212) } else { rgb(0xfee2e2) })
                                        .border_1()
                                        .border_color(if theme.is_dark() { rgb(0x7f1d1d) } else { rgb(0xfca5a5) })
                                        .hover(|s| s.bg(if theme.is_dark() { rgb(0x4c1d1d) } else { rgb(0xfecaca) }))
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(if theme.is_dark() { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, _, cx| {
                                                this.cancel_all_transfers(cx);
                                            }),
                                        )
                                        .child(render_icon("icons/close.svg", 10.0, if theme.is_dark() { 0xfca5a5 } else { 0xb91c1c }))
                                        .child(t.btn_abort_transfer()),
                                )
                            })
                            // Toggle transfer log button
                            .child(
                                div()
                                    .id("btn-toggle-transfer-log")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .px_2()
                                    .h(px(18.0))
                                    .rounded(px(3.0))
                                    .bg(if show_log { theme.button_active_bg() } else { theme.button_bg() })
                                    .border_1()
                                    .border_color(if show_log { theme.button_active_border() } else { theme.border() })
                                    .hover(|s| s.bg(theme.button_hover_bg()))
                                    .text_xs()
                                    .text_color(if show_log { theme.text_accent() } else { theme.text_secondary() })
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.show_transfer_log = !this.show_transfer_log;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/file-text.svg", 10.0, if show_log { theme.c_icon_active() } else { theme.c_icon_secondary() }))
                                    .child(if show_log { t.btn_hide_log() } else { t.btn_show_log() }),
                            )
                            // Close queue panel
                            .child(
                                div()
                                    .id("btn-close-queue-panel")
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
                                            this.show_queue = false;
                                            cx.notify();
                                        }),
                                    )
                                    .child(render_icon("icons/close.svg", 10.0, theme.c_icon_secondary())),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .w_full()
                    .overflow_hidden()
                    .child(
                        div()
                            .when(!show_log, |el| el.w_full())
                            .when(show_log, |el| el.w(gpui::DefiniteLength::Fraction(0.55)))
                            .h_full()
                            .overflow_hidden()
                            .child(self.render_transfers_list(cx)),
                    )
                    .when(show_log, |el| {
                        el.child(div().w(px(1.0)).h_full().bg(theme.border()))
                            .child(
                                div()
                                    .w(gpui::DefiniteLength::Fraction(0.45))
                                    .h_full()
                                    .overflow_hidden()
                                    .child(self.render_server_console()),
                            )
                    }),
            )
    }
}

fn compute_dir_stats(dir_id: usize, all_items: &[&TransferQueueItem]) -> (u64, u64, f64, bool, bool) {
    let mut transferred = 0u64;
    let mut total = 0u64;
    let mut max_speed = 0.0f64;
    let mut is_running = false;
    let mut all_completed = true;

    let children: Vec<&TransferQueueItem> = all_items
        .iter()
        .filter(|i| i.parent_id == Some(dir_id))
        .copied()
        .collect();

    if children.is_empty() {
        if let Some(item) = all_items.iter().find(|i| i.id == dir_id) {
            return (
                item.transferred_bytes,
                item.size_bytes,
                item.speed_mbps,
                item.status == TransferStatus::Transferring,
                item.status == TransferStatus::Completed,
            );
        }
        return (0, 0, 0.0, false, false);
    }

    for child in children {
        if child.is_dir {
            let (c_trans, c_tot, c_spd, c_run, c_comp) = compute_dir_stats(child.id, all_items);
            transferred += c_trans;
            total += c_tot;
            if c_spd > max_speed {
                max_speed = c_spd;
            }
            if c_run {
                is_running = true;
            }
            if !c_comp {
                all_completed = false;
            }
        } else {
            transferred += child.transferred_bytes;
            total += child.size_bytes;
            if child.speed_mbps > max_speed {
                max_speed = child.speed_mbps;
            }
            if child.status == TransferStatus::Transferring {
                is_running = true;
            }
            if child.status != TransferStatus::Completed {
                all_completed = false;
            }
        }
    }

    (transferred, total, max_speed, is_running, all_completed)
}

fn render_progress_bar_cell(
    pct: f32,
    text: String,
    is_running: bool,
    is_completed: bool,
    is_failed: bool,
    theme: &Theme,
) -> impl IntoElement {
    let is_dark = theme.is_dark();
    div()
        .relative()
        .w(px(150.0))
        .flex_shrink_0()
        .h(px(16.0))
        .rounded(px(3.0))
        .bg(if is_failed {
            if is_dark { rgb(0x451a1a) } else { rgb(0xfee2e2) }
        } else {
            theme.progress_bg()
        })
        .border_1()
        .border_color(if is_failed {
            if is_dark { rgb(0x7f1d1d) } else { rgb(0xfca5a5) }
        } else {
            theme.progress_border()
        })
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .bottom_0()
                .w(gpui::DefiniteLength::Fraction((pct / 100.0).clamp(0.0, 1.0)))
                .bg(rgb(if is_failed {
                    if is_dark { 0x7f1d1d } else { 0xfca5a5 }
                } else if is_completed {
                    if is_dark { 0x15803d } else { 0x86efac }
                } else if is_running {
                    if is_dark { 0x2563eb } else { 0x93c5fd }
                } else {
                    if is_dark { 0x2a2a30 } else { 0xe4e4e7 }
                })),
        )
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .flex()
                .items_center()
                .justify_center()
                .px_1()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(if is_failed {
                    if is_dark { rgb(0xfca5a5) } else { rgb(0xb91c1c) }
                } else {
                    theme.text_primary()
                })
                .overflow_hidden()
                .whitespace_nowrap()
                .child(text),
        )
}

impl RusterView {
    fn render_queue_item_tree(
        &self,
        item: &TransferQueueItem,
        all_items: &[&TransferQueueItem],
        depth: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = self.theme();
        let is_dark = theme.is_dark();
        let item_id = item.id;
        let is_dir = item.is_dir;
        let is_expanded = self.expanded_folders.contains(&item_id);

        let children: Vec<&TransferQueueItem> = all_items
            .iter()
            .filter(|i| i.parent_id == Some(item_id))
            .copied()
            .collect();
        let has_children = !children.is_empty();

        let (transferred, total, max_speed, is_running, is_completed) = if is_dir || has_children {
            compute_dir_stats(item_id, all_items)
        } else {
            (
                item.transferred_bytes,
                item.size_bytes,
                item.speed_mbps,
                item.status == TransferStatus::Transferring,
                item.status == TransferStatus::Completed,
            )
        };

        let is_failed = item.status == TransferStatus::Failed;

        let pct = if total > 0 {
            ((transferred as f64 / total as f64) * 100.0).clamp(0.0, 100.0) as f32
        } else if is_completed {
            100.0
        } else {
            0.0
        };

        let speed_str = if max_speed > 0.05 {
            format!("{:.1} MB/s", max_speed)
        } else if is_completed {
            "Hotovo".to_string()
        } else if is_failed {
            "Chyba".to_string()
        } else {
            "Čeká...".to_string()
        };

        let progress_text = if total > 0 {
            format!("{:.0}% • {}/{}", pct, format_bytes(transferred), format_bytes(total))
        } else {
            format!("{:.0}%", pct)
        };

        let indent_px = depth * 14;
        let (raw_icon, default_color) = get_file_icon(&item.filename, is_dir, false);
        let icon_path = raw_icon;
        let icon_color = theme.adjust_file_icon_color(default_color, is_dir);

        div()
            .flex()
            .flex_col()
            .gap_0p5()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(22.0))
                    .px_2()
                    .rounded(px(3.0))
                    .bg(if is_running {
                        if is_dark { rgb(0x064e3b) } else { rgb(0xf0fdf4) }
                    } else {
                        theme.bottom_panel_bg()
                    })
                    .border_1()
                    .border_color(if is_running {
                        if is_dark { rgb(0x059669) } else { rgb(0xbbf7d0) }
                    } else {
                        theme.row_border()
                    })
                    .text_xs()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(60.0))
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .overflow_hidden()
                            .pl(px(indent_px as f32))
                            .when(is_dir || has_children, |el| {
                                el.child(
                                    div()
                                        .cursor_pointer()
                                        .w(px(14.0))
                                        .h(px(14.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                            if this.expanded_folders.contains(&item_id) {
                                                this.expanded_folders.remove(&item_id);
                                            } else {
                                                this.expanded_folders.insert(item_id);
                                            }
                                            cx.notify();
                                        }))
                                        .child(render_icon(
                                            if is_expanded { "icons/chevron-down.svg" } else { "icons/chevron-right.svg" },
                                            10.0,
                                            theme.c_icon_default(),
                                        )),
                                )
                            })
                            .when(!is_dir && !has_children, |el| {
                                el.child(div().w(px(14.0)).h(px(14.0)))
                            })
                            .child(render_icon(icon_path, 12.0, icon_color))
                            .child(
                                div()
                                    .font_weight(if is_dir { gpui::FontWeight::SEMIBOLD } else { gpui::FontWeight::NORMAL })
                                    .text_color(if is_dir { theme.row_text_dir() } else { theme.text_primary() })
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .whitespace_nowrap()
                                    .child(item.filename.clone()),
                            ),
                    )
                    .child(render_progress_bar_cell(
                        pct,
                        progress_text,
                        is_running,
                        is_completed,
                        is_failed,
                        &theme,
                    ))
                    .child(
                        div()
                            .w(px(60.0))
                            .flex_shrink_0()
                            .text_right()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(if is_running { rgb(0x16a34a) } else { theme.text_muted() })
                            .child(speed_str),
                    ),
            )
            .when(is_expanded && (is_dir || has_children), |el| {
                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .children(children.into_iter().map(|child| {
                            self.render_queue_item_tree(child, all_items, depth + 1, cx)
                        })),
                )
            })
    }

    fn render_transfers_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let mut batch_map: std::collections::BTreeMap<usize, Vec<&TransferQueueItem>> = std::collections::BTreeMap::new();
        let mut batch_ids = Vec::new();

        for item in &self.transfer_queue {
            let bid = item.batch_id;
            if !batch_map.contains_key(&bid) {
                batch_ids.push(bid);
            }
            batch_map.entry(bid).or_default().push(item);
        }

        if batch_ids.is_empty() {
            return div()
                .id("empty-transfers")
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(theme.bottom_panel_bg())
                .text_xs()
                .text_color(theme.text_muted())
                .child("Žádné aktivní přenosy souborů.");
        }

        div()
            .id("bottom-transfers-scroll")
            .size_full()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .p_1p5()
            .gap_1()
            .bg(theme.bottom_panel_bg())
            .children(batch_ids.into_iter().map(|bid| {
                let items = batch_map.get(&bid).cloned().unwrap_or_default();
                let root_items: Vec<&TransferQueueItem> = items.iter().filter(|i| i.parent_id.is_none()).copied().collect();
                let total_items_count = items.len();
                let is_multi = total_items_count > 1 || root_items.iter().any(|i| i.is_dir);
                let is_expanded = self.expanded_batches.contains(&bid);

                let total_bytes: u64 = items.iter().filter(|i| !i.is_dir).map(|i| i.size_bytes).sum();
                let transferred_bytes: u64 = items.iter().filter(|i| !i.is_dir).map(|i| i.transferred_bytes).sum();
                let completed_count = items.iter().filter(|i| !i.is_dir && i.status == TransferStatus::Completed).count();
                let total_file_count = items.iter().filter(|i| !i.is_dir).count().max(1);

                let active_items: Vec<&TransferQueueItem> = items.iter().copied().filter(|i| i.status == TransferStatus::Transferring && !i.is_dir).collect();
                let active_speed: f64 = active_items.iter().map(|i| i.speed_mbps).sum();
                let batch_pct = if total_bytes > 0 {
                    ((transferred_bytes as f64 / total_bytes as f64) * 100.0).clamp(0.0, 100.0) as f32
                } else {
                    ((completed_count as f64 / total_file_count as f64) * 100.0).clamp(0.0, 100.0) as f32
                };
                let direction = items.first().map(|i| i.direction).unwrap_or(TransferDirection::Download);
                let dir_icon = match direction {
                    TransferDirection::Download => "icons/arrow-down.svg",
                    TransferDirection::Upload => "icons/arrow-up.svg",
                };
                let dir_color = match direction {
                    TransferDirection::Download => 0x16a34a,
                    TransferDirection::Upload => 0x2563eb,
                };

                let title_text = match direction {
                    TransferDirection::Download => format!("Stahování ({} položek)", total_items_count),
                    TransferDirection::Upload => format!("Nahrávání ({} položek)", total_items_count),
                };

                let active_subtitle = if active_items.len() > 1 {
                    Some(format!("- {} paralelně", active_items.len()))
                } else if let Some(first) = active_items.first() {
                    Some(format!("- {}", first.filename))
                } else {
                    None
                };

                let speed_str = if active_speed > 0.05 {
                    format!("{:.1} MB/s", active_speed)
                } else {
                    "Čeká...".to_string()
                };

                let batch_progress_text = if total_bytes > 0 {
                    format!("{:.0}% • {} / {}", batch_pct, format_bytes(transferred_bytes), format_bytes(total_bytes))
                } else {
                    format!("{:.0}% • {}/{}", batch_pct, completed_count, total_file_count)
                };

                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        // Batch summary row
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .h(px(26.0))
                            .px_2()
                            .rounded(px(4.0))
                            .bg(theme.header_focused_bg())
                            .border_1()
                            .border_color(theme.border())
                            .text_xs()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(60.0))
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .overflow_hidden()
                                    .when(is_multi, |el| {
                                        el.child(
                                            div()
                                                .cursor_pointer()
                                                .w(px(14.0))
                                                .h(px(14.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                    if this.expanded_batches.contains(&bid) {
                                                        this.expanded_batches.remove(&bid);
                                                    } else {
                                                        this.expanded_batches.insert(bid);
                                                    }
                                                    cx.notify();
                                                }))
                                                .child(render_icon(
                                                    if is_expanded { "icons/chevron-down.svg" } else { "icons/chevron-right.svg" },
                                                    11.0,
                                                    theme.c_icon_secondary(),
                                                )),
                                        )
                                    })
                                    .child(render_icon(dir_icon, 13.0, dir_color))
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary())
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .whitespace_nowrap()
                                            .child(title_text),
                                    )
                                    .when_some(active_subtitle, |el, sub| {
                                        el.child(
                                            div()
                                                .text_color(theme.text_secondary())
                                                .overflow_hidden()
                                                .text_ellipsis()
                                                .whitespace_nowrap()
                                                .child(sub),
                                        )
                                    }),
                            )
                            .child(render_progress_bar_cell(
                                batch_pct,
                                batch_progress_text,
                                active_speed > 0.05,
                                batch_pct >= 100.0,
                                false,
                                &theme,
                            ))
                            .child(
                                div()
                                    .w(px(60.0))
                                    .flex_shrink_0()
                                    .text_right()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(if active_speed > 0.05 { theme.text_accent() } else { theme.text_muted() })
                                    .child(speed_str),
                            ),
                    )
                    .when(is_expanded, |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .children(root_items.into_iter().filter(|i| i.status != TransferStatus::Completed).map(|root_item| {
                                    self.render_queue_item_tree(root_item, &items, 1, cx)
                                })),
                        )
                    })
            }))
    }

    fn render_server_console(&self) -> impl IntoElement {
        let theme = self.theme();
        let is_dark = theme.is_dark();
        if self.server_log.is_empty() {
            return div()
                .id("empty-server-console")
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(theme.console_bg())
                .text_xs()
                .text_color(theme.text_muted())
                .child("Konzole serveru je prázdná. Zde se zobrazí odeslané příkazy a odpovědi.");
        }

        div()
            .id("bottom-console-scroll")
            .size_full()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&self.console_scroll)
            .p_2()
            .gap_0p5()
            .bg(theme.console_bg())
            .children(self.server_log.iter().map(|entry| {
                let (icon, icon_color, text_color, is_bold) = match entry.kind {
                    ServerLogKind::Command => ("icons/arrow-right.svg", if is_dark { 0x60a5fa } else { 0x2563eb }, if is_dark { 0x60a5fa } else { 0x1d4ed8 }, false),
                    ServerLogKind::Response => ("icons/arrow-left.svg", if is_dark { 0x4ade80 } else { 0x16a34a }, if is_dark { 0xf4f4f5 } else { 0x18181b }, false),
                    ServerLogKind::Status => ("icons/info.svg", if is_dark { 0x94a3b8 } else { 0x64748b }, if is_dark { 0xa1a1aa } else { 0x52525b }, false),
                    ServerLogKind::Error => ("icons/alert-circle.svg", if is_dark { 0xf87171 } else { 0xdc2626 }, if is_dark { 0xf87171 } else { 0xb91c1c }, true),
                };

                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_1()
                    .py_0p5()
                    .rounded(px(2.0))
                    .hover(|s| s.bg(theme.button_hover_bg()))
                    .text_xs()
                    .child(
                        div()
                            .text_color(theme.text_muted())
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(format!("[{}]", entry.time)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(14.0))
                            .h(px(14.0))
                            .flex_shrink_0()
                            .child(render_icon(icon, 13.0, icon_color)),
                    )
                    .child(
                        div()
                            .font_weight(if is_bold {
                                gpui::FontWeight::SEMIBOLD
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .text_color(rgb(text_color))
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(entry.message.clone()),
                    )
            }))
    }
}
