mod config;
mod models;
mod paste;
mod process;
mod record;

use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use config::ConfigManager;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use process::Processor;
use record::{Recorder, RecordingHandle};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;
use tray_icon::menu::{AboutMetadataBuilder, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIconBuilder, TrayIconEvent};

const APP_NAME: &str = "whisp";
const DEFAULT_LOG_LEVEL: &str = "info";
const ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    // Initialize the logger
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("WHISP_LOG")
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_LEVEL)),
        )
        .init();

    // Load config
    let config_manager = ConfigManager::new()?;
    let config = Arc::new(RwLock::new(config_manager.load()?));
    // save back the config to create the file if it doesn't exist
    config_manager.save(&config.read().unwrap())?;

    // Set up hotkey
    let hotkey_manager = GlobalHotKeyManager::new().context("Failed to create hotkey manager")?;
    hotkey_manager
        .register(config.read().unwrap().hotkey())
        .context("Failed to register hotkey")?;

    // Set up recorder
    let recorder = Recorder::new();
    let mut active_recording: Option<RecordingHandle> = None;

    // Set up processor for handling audio data
    let processor = Processor::new(config.clone())?;

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

    // Set up the event loop
    let mut icon_tray = None;
    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let hotkey_channel = GlobalHotKeyEvent::receiver();

    EventLoopBuilder::new()
        .build()
        .run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

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
                // Handle other menu events
            }

            #[expect(clippy::redundant_pattern_matching)]
            if let Ok(_) = tray_channel.try_recv() {
                // Handle tray icon events
            }

            if let Ok(event) = hotkey_channel.try_recv() {
                if event.id() == config.read().unwrap().hotkey().id()
                    && event.state() == HotKeyState::Pressed
                {
                    match active_recording.take() {
                        Some(mut recording) => match recording.finish() {
                            Ok(Some(data)) => {
                                if processor.submit_audio(data).is_err() {
                                    error!("Failed to submit audio to processor");
                                }
                            }
                            Ok(None) => {
                                warn!("Recording finished but no data was recorded");
                            }
                            Err(e) => {
                                error!(error = ?e, "Failed to finish recording");
                            }
                        },
                        None => match recorder.start_recording() {
                            Ok(handle) => {
                                active_recording = Some(handle);
                            }
                            Err(e) => {
                                error!(error = ?e, "Failed to start recording");
                            }
                        },
                    }
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
