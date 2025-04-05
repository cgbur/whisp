use std::sync::Arc;
use std::thread::sleep;

use anyhow::{Context, Result};
use arboard::Clipboard;
use enigo::Enigo;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use parking_lot::RwLock;
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tracing::{error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tray_icon::menu::{AboutMetadataBuilder, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIconBuilder, TrayIconEvent};
use whisp::config::ConfigManager;
use whisp::event::WhispEvent;
use whisp::icon::MicState;
use whisp::notify::NotificationLayer;
use whisp::process::AudioPipeline;
use whisp::record::{Recorder, RecordingHandle};
use whisp::{DEFAULT_LOG_LEVEL, VERSION};

fn main() -> Result<()> {
    // Initialize the logger
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("WHISP_LOG")
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_LEVEL)),
        )
        .finish()
        .with(NotificationLayer::new())
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

    // Set up keyboard and clipboard interaction
    let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
    let mut clipboard = Clipboard::new()?;

    // Create the tray menu
    let tray_menu = Menu::new();
    let icon_quit = MenuItem::new("Quit", true, None);
    let icon_copy_config = MenuItem::new("Copy config path", true, None);
    tray_menu.append_items(&[
        // the name of the app
        &MenuItem::new("Whisp", false, None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::about(
            None,
            Some(
                AboutMetadataBuilder::new()
                    .version(Some(VERSION.to_owned()))
                    .build(),
            ),
        ),
        &icon_copy_config,
        &PredefinedMenuItem::separator(),
        &icon_quit,
    ])?;

    // Set up the event loop
    let mut icon_tray = None;

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let hotkey_channel = GlobalHotKeyEvent::receiver();

    let event_loop: EventLoop<WhispEvent> = EventLoopBuilder::with_user_event().build();
    let event_sender = event_loop.create_proxy();

    // Set up processor for handling audio data async operations
    let audio_pipeline = AudioPipeline::new(config.clone(), event_sender.clone())?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::NewEvents(StartCause::Init) = event {
            // We create the icon once the event loop is actually running
            // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90

            icon_tray.replace(
                TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu.clone()))
                    .with_tooltip("whisp - speech to text")
                    .with_icon(MicState::Idle.icon())
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

            info!("Whisp ready");
        }

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == icon_quit.id() {
                icon_tray.take();
                *control_flow = ControlFlow::Exit;
            } else if event.id == icon_copy_config.id() {
                if let Err(e) =
                    clipboard.set_text(config_manager.config_path().to_string_lossy().into_owned())
                {
                    error!("Failed to copy config path to clipboard: {}", e);
                }
            }
        }

        #[expect(clippy::redundant_pattern_matching)]
        if let Ok(_) = tray_channel.try_recv() {
            // Handle tray icon events
        }

        // Handle user provided events
        if let Event::UserEvent(event) = event {
            match event {
                WhispEvent::StateChanged(state) => {
                    info!(state = ?state, "State changed");
                    icon_tray.as_ref().map(|i| i.set_icon(Some(state.icon())));
                }
                WhispEvent::TranscriptReady(text) => {
                    // Set the state to idle so it goes back to being inactive
                    // after this transcription
                    event_sender
                        .send_event(WhispEvent::StateChanged(MicState::Idle))
                        .ok();

                    let config = config.read();
                    info!(
                        auto_paste = config.auto_paste(),
                        restore_clipboard = config.restore_clipboard(),
                        "Handling transcription"
                    );
                    let restore = config.auto_paste() && config.restore_clipboard();
                    let previous = if restore {
                        match clipboard.get_text() {
                            Ok(text) => Some(text),
                            Err(e) => {
                                warn!("Failed to get clipboard text: {}", e);
                                None
                            }
                        }
                    } else {
                        None
                    };

                    // Copy the transcription to the clipboard
                    if let Err(e) = clipboard.set_text(&text) {
                        warn!("Failed to set clipboard text: {}", e);
                    }

                    if config.auto_paste() {
                        // Paste the transcription
                        if let Err(e) = paste(&mut enigo) {
                            warn!("Failed to paste transcription: {}", e);
                        }
                        if let Some(previous) = previous {
                            // Restore the previous clipboard contents
                            if let Err(e) = clipboard.set_text(&previous) {
                                warn!("Failed to restore clipboard text: {}", e);
                            }
                        }
                    }
                }
                WhispEvent::AudioError(_) => {
                    warn!("Audio processing error received, author has not yet implemented this");
                }
            };
        }

        // Handle hotkey events
        if let Ok(event) = hotkey_channel.try_recv() {
            if event.id() == config.read().hotkey().id() && event.state() == HotKeyState::Pressed {
                let mic_state = match active_recording.take() {
                    Some(mut recording) => match recording.finish() {
                        Ok(Some(data)) => match audio_pipeline.submit(data) {
                            Ok(whisp::process::SubmitResult::Discarded) => MicState::Idle,
                            Ok(whisp::process::SubmitResult::Sent) => MicState::Processing,
                            Err(e) => {
                                error!("Failed to submit audio to processor: {:?}", e);
                                MicState::Idle
                            }
                        },
                        Ok(None) => {
                            warn!("Recording finished but no data was recorded");
                            MicState::Idle
                        }
                        Err(e) => {
                            error!(error = ?e, "Failed to finish recording");
                            MicState::Idle
                        }
                    },
                    None => match recorder.start_recording(event_sender.clone()) {
                        Ok(handle) => {
                            active_recording = Some(handle);
                            MicState::Activating
                        }
                        Err(e) => {
                            error!("Failed to start recording: {:?}", e);
                            MicState::Idle
                        }
                    },
                };
                event_sender
                    .send_event(WhispEvent::StateChanged(mic_state))
                    .ok();
            }
        }
    });
}

fn paste(enigo: &mut Enigo) -> anyhow::Result<()> {
    use enigo::Direction::{Click, Press, Release};
    use enigo::{Key, Keyboard};

    #[cfg(target_os = "macos")]
    let paste_modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let paste_modifier = Key::Control;

    const SLEEP_TIME: std::time::Duration = std::time::Duration::from_millis(10);
    enigo.key(paste_modifier, Press)?;
    sleep(SLEEP_TIME);
    enigo.key(Key::Unicode('v'), Click)?;
    sleep(SLEEP_TIME);
    enigo.key(paste_modifier, Release)?;

    Ok(())
}
