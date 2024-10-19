use std::path::Path;
use std::sync::{Arc, LazyLock};

use anyhow::{Context, Result};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use parking_lot::{Mutex, RwLock};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use tray_icon::menu::{AboutMetadataBuilder, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};
use whisp::config::ConfigManager;
use whisp::event::UserEvent;
use whisp::icon::MicState;
use whisp::process::Processor;
use whisp::record::{Recorder, RecordingHandle};
use whisp::{DEFAULT_LOG_LEVEL, VERSION};

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
    config_manager.save(&config.read())?;

    // Set up hotkey
    let hotkey_manager = GlobalHotKeyManager::new().context("Failed to create hotkey manager")?;
    hotkey_manager
        .register(config.read().hotkey())
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

    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
    let event_sender = event_loop.create_proxy();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::NewEvents(StartCause::Init) = event {
            // We create the icon once the event loop is actually running
            // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90

            icon_tray.replace(
                TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu.clone()))
                    .with_tooltip("whisp - speech to text")
                    .with_icon(MicState::Inactive.icon())
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

        if let Event::UserEvent(event) = event {
            match event {
                UserEvent::SetIcon(s) => icon_tray.as_ref().map(|i| i.set_icon(Some(s.icon()))),
            };
        }

        if let Ok(event) = hotkey_channel.try_recv() {
            if event.id() == config.read().hotkey().id() && event.state() == HotKeyState::Pressed {
                match active_recording.take() {
                    Some(mut recording) => {
                        event_sender
                            .send_event(UserEvent::SetIcon(MicState::Inactive))
                            .ok();
                        match recording.finish() {
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
                        }
                    }
                    None => {
                        event_sender
                            .send_event(UserEvent::SetIcon(MicState::Activating))
                            .ok();
                        match recorder.start_recording(event_sender.clone()) {
                            Ok(handle) => {
                                active_recording = Some(handle);
                            }
                            Err(e) => {
                                error!(error = ?e, "Failed to start recording");
                            }
                        }
                    }
                }
            }
        }
    });
}
