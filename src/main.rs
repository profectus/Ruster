mod app;
pub mod archive;
mod config;
mod i18n;
mod icons;
mod model;
mod network;
mod theme;
mod ui;
pub mod sync;
pub mod update;

use gpui::{
    actions, px, size, App, AppContext, Application, Bounds, KeyBinding, SharedString,
    TitlebarOptions, WindowBounds, WindowOptions,
};
use icons::RusterAssets;
use app::RusterView;
use ui::menus::install_app_menus;

pub const APP_ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
pub fn set_macos_window_appearance(is_dark: bool) {
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CString;

    let appearance_name = if is_dark {
        CString::new("NSAppearanceNameDarkAqua").unwrap()
    } else {
        CString::new("NSAppearanceNameAqua").unwrap()
    };

    unsafe {
        let ns_str: *mut objc::runtime::Object = msg_send![
            class!(NSString),
            stringWithUTF8String: appearance_name.as_ptr()
        ];
        if ns_str.is_null() {
            return;
        }
        let appearance: *mut objc::runtime::Object = msg_send![
            class!(NSAppearance),
            appearanceNamed: ns_str
        ];
        if appearance.is_null() {
            return;
        }
        let nsapp: *mut objc::runtime::Object = msg_send![class!(NSApplication), sharedApplication];
        if !nsapp.is_null() {
            let _: () = msg_send![nsapp, setAppearance: appearance];
            let windows: *mut objc::runtime::Object = msg_send![nsapp, windows];
            if !windows.is_null() {
                let count: usize = msg_send![windows, count];
                for i in 0..count {
                    let win: *mut objc::runtime::Object = msg_send![windows, objectAtIndex: i];
                    if !win.is_null() {
                        let _: () = msg_send![win, setAppearance: appearance];
                    }
                }
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_macos_window_appearance(_is_dark: bool) {}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
pub fn set_macos_dock_icon(png_bytes: &[u8]) {
    use objc::{class, msg_send, sel, sel_impl};
    unsafe {
        let nsdata: *mut objc::runtime::Object = msg_send![
            class!(NSData),
            dataWithBytes: png_bytes.as_ptr()
            length: png_bytes.len()
        ];
        if nsdata.is_null() {
            return;
        }
        let nsimage: *mut objc::runtime::Object = msg_send![class!(NSImage), alloc];
        let nsimage: *mut objc::runtime::Object = msg_send![nsimage, initWithData: nsdata];
        if !nsimage.is_null() {
            let nsapp: *mut objc::runtime::Object = msg_send![class!(NSApplication), sharedApplication];
            if !nsapp.is_null() {
                let _: () = msg_send![nsapp, setApplicationIconImage: nsimage];
            }
            let _: () = msg_send![nsimage, release];
        }
    }
}

actions!(
    ruster,
    [
        Quit,
        OpenSiteManager,
        ToggleDualPane,
        SwapPanes,
        ToggleLocalTree,
        ToggleRemoteTree,
        ToggleHiddenFiles,
        ToggleTransferQueue,
        ToggleFunctionBar,
        ToggleQuickFilter,
        RefreshPanes,
        SwitchLanguage,
        ToggleTheme,
        OpenSelected,
        EditSelected,
        CopySelected,
        MoveSelected,
        NewFolderModal,
        RenameModal,
        DeleteModal,
        PermissionsModal,
        SynchronizeModal,
        RevealInFinder,
        OpenInTerminal,
        SelectAllItems,
        GoToFolderModal,
        CompressModal,
        ExtractModal,
        CheckForUpdates,
    ]
);

fn main() {
    Application::new().with_assets(RusterAssets).run(|cx: &mut App| {
        #[cfg(target_os = "macos")]
        set_macos_dock_icon(APP_ICON_PNG);

        cx.on_action(|_: &Quit, cx| {
            crate::config::cleanup_remote_edit_temp();
            cx.quit();
        });
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                crate::config::cleanup_remote_edit_temp();
                cx.quit();
            }
        })
        .detach();
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-o", OpenSiteManager, None),
            KeyBinding::new("cmd-e", EditSelected, None),
            KeyBinding::new("f4", EditSelected, None),
            KeyBinding::new("cmd-1", ToggleDualPane, None),
            KeyBinding::new("cmd-u", SwapPanes, None),
            KeyBinding::new("cmd-2", ToggleLocalTree, None),
            KeyBinding::new("cmd-3", ToggleRemoteTree, None),
            KeyBinding::new("cmd-shift-.", ToggleHiddenFiles, None),
            KeyBinding::new("cmd-j", ToggleTransferQueue, None),
            KeyBinding::new("cmd-f", ToggleQuickFilter, None),
            KeyBinding::new("cmd-g", GoToFolderModal, None),
            KeyBinding::new("cmd-r", RefreshPanes, None),
            KeyBinding::new("cmd-l", SwitchLanguage, None),
            KeyBinding::new("cmd-shift-t", ToggleTheme, None),
            KeyBinding::new("cmd-c", CopySelected, None),
            KeyBinding::new("cmd-shift-n", NewFolderModal, None),
            KeyBinding::new("cmd-backspace", DeleteModal, None),
            KeyBinding::new("cmd-i", PermissionsModal, None),
            KeyBinding::new("cmd-s", SynchronizeModal, None),
            KeyBinding::new("cmd-shift-r", RevealInFinder, None),
            KeyBinding::new("cmd-alt-t", OpenInTerminal, None),
            KeyBinding::new("cmd-a", SelectAllItems, None),
            KeyBinding::new("alt-f5", CompressModal, None),
            KeyBinding::new("alt-f6", ExtractModal, None),
        ]);
        let state = crate::config::load_app_state();
        let is_dark = state.theme.is_dark();
        set_macos_window_appearance(is_dark);
        install_app_menus(cx, state.locale);
        let display = cx.primary_display();
        let max_w = display
            .as_ref()
            .map(|d| (f32::from(d.bounds().size.width) - 40.0).max(600.0))
            .unwrap_or(2560.0);
        let max_h = display
            .as_ref()
            .map(|d| (f32::from(d.bounds().size.height) - 80.0).max(400.0))
            .unwrap_or(1440.0);
        let width = state.window_width.clamp(600.0, max_w);
        let height = state.window_height.clamp(400.0, max_h);
        let bounds = Bounds::centered(None, size(px(width), px(height)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("Ruster")),
                    ..Default::default()
                }),
                is_resizable: true,
                ..Default::default()
            },
            |_, cx| cx.new(RusterView::new),
        )
        .unwrap();
        set_macos_window_appearance(is_dark);
        cx.activate(true);
    });
}
