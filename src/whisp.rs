use std::path::Path;

use anyhow::Context;
use global_hotkey::hotkey::{HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tracing::info;
use tracing_subscriber::EnvFilter;
use tray_icon::menu::{AboutMetadataBuilder, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIconBuilder, TrayIconEvent};

const DEFAULT_LOG_LEVEL: &str = "info";
const ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> anyhow::Result<()> {
    // Initialize the logger
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("WHISP_LOG")
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_LEVEL)),
        )
        .init();

    // Create the tray menu
    let tray_menu = Menu::new();
    let icon_quit = MenuItem::new("Quit", true, None);
    tray_menu.append_items(&[
        &PredefinedMenuItem::about(
            None,
            Some(
                AboutMetadataBuilder::new()
                    .version(Some(VERSION.to_owned()))
                    .build(),
            ),
        ),
        &PredefinedMenuItem::separator(),
        &icon_quit,
    ])?;

    // Register the global hotkey
    let hotkey_manager = GlobalHotKeyManager::new().context("Failed to create hotkey manager")?;
    let hotkey = HotKey::new(
        Some(Modifiers::META | Modifiers::SHIFT),
        global_hotkey::hotkey::Code::Semicolon,
    );
    hotkey_manager
        .register(hotkey)
        .context("Failed to register hotkey")?;

    // Set up the event loop
    let mut icon_tray = None;
    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let hotkey_channel = GlobalHotKeyEvent::receiver();

    EventLoopBuilder::new()
        .build()
        .run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            info!("Tick");

            if let Event::NewEvents(StartCause::Init) = event {
                let icon = load_icon(Path::new(ICON_PATH));

                // We create the icon once the event loop is actually running
                // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                icon_tray = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu.clone()))
                        .with_tooltip("whisp - speech to text")
                        .with_icon(icon)
                        .build()
                        .unwrap(),
                );

                // We have to request a redraw here to have the icon actually show up.
                // Tao only exposes a redraw method on the Window so we use core-foundation directly.
                #[cfg(target_os = "macos")]
                unsafe {
                    use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopWakeUp};

                    let rl = CFRunLoopGetMain();
                    CFRunLoopWakeUp(rl);
                }
            }

            if let Ok(event) = menu_channel.try_recv() {
                if event.id == icon_quit.id() {
                    icon_tray.take();
                    *control_flow = ControlFlow::Exit;
                }
                println!("{event:?}");
            }

            if let Ok(event) = tray_channel.try_recv() {
                println!("{event:?}");
            }

            if let Ok(event) = hotkey_channel.try_recv() {
                println!("{event:?}");
                if event.id() == hotkey.id() {
                    println!("Hotkey pressed!");
                }
            }
        })
}

fn load_icon(path: &Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
