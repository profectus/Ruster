use gpui::{rgb, rgba, Rgba};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[allow(dead_code)]
impl ThemeMode {
    pub fn is_dark(&self) -> bool {
        matches!(self, ThemeMode::Dark)
    }

    pub fn toggle(&self) -> Self {
        match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::Light,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub mode: ThemeMode,
}

#[allow(dead_code)]
impl Theme {
    pub const fn new(mode: ThemeMode) -> Self {
        Self { mode }
    }

    #[inline(always)]
    pub fn is_dark(&self) -> bool {
        self.mode.is_dark()
    }

    // --- Backgrounds ---
    pub fn root_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x18181b)
        } else {
            rgb(0xffffff)
        }
    }

    pub fn pane_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x18181b)
        } else {
            rgb(0xffffff)
        }
    }

    pub fn top_bar_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x1e1e22)
        } else {
            rgb(0xf4f4f5)
        }
    }

    pub fn header_focused_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x242429)
        } else {
            rgb(0xf8fafc)
        }
    }

    pub fn header_unfocused_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x1d1d21)
        } else {
            rgb(0xfafafa)
        }
    }

    pub fn tree_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x19191d)
        } else {
            rgb(0xfafafa)
        }
    }

    pub fn tree_node_active_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x1e3a5f)
        } else {
            rgb(0xeff6ff)
        }
    }

    pub fn tree_node_normal_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x19191d)
        } else {
            rgb(0xfafafa)
        }
    }

    pub fn filter_bar_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x212126)
        } else {
            rgb(0xf8fafc)
        }
    }

    pub fn bottom_panel_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x161619)
        } else {
            rgb(0xfcfcfd)
        }
    }

    pub fn bottom_bar_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x1e1e22)
        } else {
            rgb(0xf4f4f5)
        }
    }

    pub fn console_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x121214)
        } else {
            rgb(0xffffff)
        }
    }

    pub fn table_header_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x222226)
        } else {
            rgb(0xf8fafc)
        }
    }

    // --- Buttons & Controls ---
    pub fn button_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x27272c)
        } else {
            rgb(0xffffff)
        }
    }

    pub fn button_hover_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x323238)
        } else {
            rgb(0xe4e4e7)
        }
    }

    pub fn button_active_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x1e3a5f)
        } else {
            rgb(0xeff6ff)
        }
    }

    pub fn button_active_border(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x3b82f6)
        } else {
            rgb(0x93c5fd)
        }
    }

    pub fn button_secondary_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x28282e)
        } else {
            rgb(0xf4f4f5)
        }
    }

    // --- Inputs ---
    pub fn input_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x141417)
        } else {
            rgb(0xffffff)
        }
    }

    pub fn input_border(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x3e3e48)
        } else {
            rgb(0xd4d4d8)
        }
    }

    pub fn input_border_focus(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x3b82f6)
        } else {
            rgb(0x2563eb)
        }
    }

    // --- Borders & Separators ---
    pub fn border(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x2e2e34)
        } else {
            rgb(0xe4e4e7)
        }
    }

    pub fn border_subtle(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x232328)
        } else {
            rgb(0xf4f4f5)
        }
    }

    pub fn border_strong(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x3e3e48)
        } else {
            rgb(0xd4d4d8)
        }
    }

    pub fn border_focused(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x3b82f6)
        } else {
            rgb(0x93c5fd)
        }
    }

    pub fn row_border(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x202024)
        } else {
            rgb(0xf4f4f5)
        }
    }

    // --- Row States ---
    pub fn row_bg_normal(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x18181b)
        } else {
            rgb(0xffffff)
        }
    }

    pub fn row_bg_selected(&self) -> Rgba {
        rgb(0x2563eb)
    }

    pub fn row_bg_space_marked(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x172554)
        } else {
            rgb(0xeff6ff)
        }
    }

    pub fn row_text_normal(&self) -> Rgba {
        if self.is_dark() {
            rgb(0xf4f4f5)
        } else {
            rgb(0x18181b)
        }
    }

    pub fn row_text_selected(&self) -> Rgba {
        rgb(0xffffff)
    }

    pub fn row_text_dir(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x60a5fa)
        } else {
            rgb(0x1d4ed8)
        }
    }

    pub fn row_text_secondary(&self) -> Rgba {
        if self.is_dark() {
            rgb(0xa1a1aa)
        } else {
            rgb(0x71717a)
        }
    }

    // --- Text ---
    pub fn text_primary(&self) -> Rgba {
        if self.is_dark() {
            rgb(0xf4f4f5)
        } else {
            rgb(0x18181b)
        }
    }

    pub fn text_secondary(&self) -> Rgba {
        if self.is_dark() {
            rgb(0xa1a1aa)
        } else {
            rgb(0x52525b)
        }
    }

    pub fn text_muted(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x71717a)
        } else {
            rgb(0xa1a1aa)
        }
    }

    pub fn text_accent(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x60a5fa)
        } else {
            rgb(0x2563eb)
        }
    }

    // --- Modals & Popovers ---
    pub fn modal_backdrop(&self) -> Rgba {
        if self.is_dark() {
            rgba(0x00000077)
        } else {
            rgba(0x00000055)
        }
    }

    pub fn modal_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x1e1e22)
        } else {
            rgb(0xffffff)
        }
    }

    pub fn modal_header_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x25252b)
        } else {
            rgb(0xf8fafc)
        }
    }

    pub fn modal_card_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x25252b)
        } else {
            rgb(0xf8fafc)
        }
    }

    pub fn modal_sidebar_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x19191d)
        } else {
            rgb(0xfafafa)
        }
    }

    pub fn popover_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x222227)
        } else {
            rgb(0xffffff)
        }
    }

    pub fn popover_item_active(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x2d2d34)
        } else {
            rgb(0xf4f4f5)
        }
    }

    // --- Transfer Progress ---
    pub fn progress_bg(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x2a2a30)
        } else {
            rgb(0xe4e4e7)
        }
    }

    pub fn progress_bar(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x3b82f6)
        } else {
            rgb(0x2563eb)
        }
    }

    pub fn progress_border(&self) -> Rgba {
        if self.is_dark() {
            rgb(0x3a3a44)
        } else {
            rgb(0xd4d4d8)
        }
    }

    // --- Colors as u32 for icons / SVG ---
    pub fn c_icon_default(&self) -> u32 {
        if self.is_dark() {
            0xa1a1aa
        } else {
            0x71717a
        }
    }

    pub fn c_icon_secondary(&self) -> u32 {
        if self.is_dark() {
            0x94a3b8
        } else {
            0x64748b
        }
    }

    pub fn c_icon_active(&self) -> u32 {
        if self.is_dark() {
            0x60a5fa
        } else {
            0x2563eb
        }
    }

    pub fn c_icon_dark(&self) -> u32 {
        if self.is_dark() {
            0xd4d4d8
        } else {
            0x3f3f46
        }
    }

    pub fn c_text_primary(&self) -> u32 {
        if self.is_dark() {
            0xf4f4f5
        } else {
            0x18181b
        }
    }

    pub fn c_text_secondary(&self) -> u32 {
        if self.is_dark() {
            0xa1a1aa
        } else {
            0x52525b
        }
    }

    pub fn c_folder(&self) -> u32 {
        if self.is_dark() {
            0x60a5fa
        } else {
            0x2563eb
        }
    }

    pub fn adjust_file_icon_color(&self, color: u32, is_dir: bool) -> u32 {
        if !self.is_dark() {
            return color;
        }
        if is_dir {
            return 0x60a5fa;
        }
        match color {
            0x71717a => 0xa1a1aa,
            0x64748b => 0x94a3b8,
            0x475569 => 0x94a3b8,
            0x3f3f46 => 0xd4d4d8,
            0x18181b => 0xf4f4f5,
            0x2563eb => 0x60a5fa,
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_toggle_and_string() {
        let light = ThemeMode::Light;
        assert_eq!(light.as_str(), "light");
        assert!(!light.is_dark());

        let dark = light.toggle();
        assert_eq!(dark, ThemeMode::Dark);
        assert_eq!(dark.as_str(), "dark");
        assert!(dark.is_dark());

        assert_eq!(dark.toggle(), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str("dark"), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_str("DARK"), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_str("light"), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str("anything_else"), ThemeMode::Light);
    }

    #[test]
    fn test_theme_color_contrast() {
        let light_theme = Theme::new(ThemeMode::Light);
        let dark_theme = Theme::new(ThemeMode::Dark);

        assert_ne!(light_theme.root_bg(), dark_theme.root_bg());
        assert_ne!(light_theme.pane_bg(), dark_theme.pane_bg());
        assert_ne!(light_theme.border(), dark_theme.border());

        // File icon adjustments in dark mode
        assert_eq!(light_theme.adjust_file_icon_color(0x71717a, false), 0x71717a);
        assert_eq!(dark_theme.adjust_file_icon_color(0x71717a, false), 0xa1a1aa);
        assert_eq!(dark_theme.adjust_file_icon_color(0x2563eb, true), 0x60a5fa);
    }
}
