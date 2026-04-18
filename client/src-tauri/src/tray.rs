//! 系统托盘模块
//!
//! 负责创建和管理系统托盘图标及菜单。

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};

/// 菜单项 ID
const MENU_ITEM_QUIT: &str = "quit";
const MENU_ITEM_SHOW: &str = "show";

/// 显示主窗口
///
/// # 参数
/// * `app` - Tauri 应用句柄
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 构建托盘菜单
///
/// # 参数
/// * `app` - Tauri 应用句柄
///
/// # 错误
/// 如果菜单创建失败，返回 Tauri 错误
fn build_tray_menu(app: &App) -> tauri::Result<Menu<tauri::Wry>> {
    let quit_item = MenuItem::with_id(app, MENU_ITEM_QUIT, "退出", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, MENU_ITEM_SHOW, "设置", true, None::<&str>)?;
    Menu::with_items(app, &[&show_item, &quit_item])
}

/// 处理托盘菜单事件
///
/// # 参数
/// * `app` - Tauri 应用句柄
/// * `event` - 菜单事件
fn on_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        MENU_ITEM_QUIT => app.exit(0),
        MENU_ITEM_SHOW => show_main_window(app),
        _ => {} // 忽略未知菜单项
    }
}

/// 处理托盘图标事件
///
/// # 参数
/// * `tray` - 托盘图标句柄
/// * `event` - 托盘事件
fn on_tray_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::DoubleClick {
        button: MouseButton::Left,
        ..
    } = event
    {
        // 仅响应左键双击显示窗口
        // 右键点击应显示托盘菜单
        show_main_window(tray.app_handle());
    }
}

/// 设置系统托盘
///
/// # 参数
/// * `app` - Tauri 应用实例
///
/// # 错误
/// 如果托盘创建失败，返回 Tauri 错误
pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let menu = build_tray_menu(app)?;

    // 安全地获取默认窗口图标
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::WindowNotFound)?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .on_menu_event(on_menu_event)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(on_tray_event)
        .build(app)?;

    Ok(())
}
