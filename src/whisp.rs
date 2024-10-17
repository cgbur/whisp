mod audio;
mod config;
mod transcribe;

use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use audio::{Recorder, RecordingHandle};
use config::{Config, ConfigManager};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use transcribe::Transcriber;
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

    let mut state = Arc::new(RwLock::new(State::new()?));
    let transcriber = Transcriber::new()?;

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
                handle_hotkey_event(event, &mut state, &transcriber);
            }
        })
}

struct State {
    hotkey_manager: GlobalHotKeyManager,
    config_manager: ConfigManager,
    config: Config,
    recorder: Recorder,
    active_recording: Option<RecordingHandle>,
    runtime: tokio::runtime::Runtime,
}

impl State {
    fn new() -> Result<Self> {
        // Load config
        let config_manager = ConfigManager::new()?;
        info!("Loaded config");
        let config = config_manager.load().unwrap();
        info!(config = ?config, "Loaded config");
        // save back the config to create the file if it doesn't exist
        config_manager.save(&config)?;

        // Set up hotkey
        let hotkey_manager =
            GlobalHotKeyManager::new().context("Failed to create hotkey manager")?;
        hotkey_manager
            .register(config.hotkey())
            .context("Failed to register hotkey")?;

        // Set up recorder
        let recorder = Recorder::new();

        // Set up tokio runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?;

        Ok(Self {
            hotkey_manager,
            config_manager,
            config,
            recorder,
            active_recording: None,
            runtime,
        })
    }
}

fn handle_hotkey_event(
    event: GlobalHotKeyEvent,
    state: &mut Arc<RwLock<State>>,
    transcriber: &Transcriber,
) {
    if event.id() == state.read().unwrap().config.hotkey().id()
        && event.state() == HotKeyState::Pressed
    {
        let mut state = state.write().unwrap();
        match state.active_recording.take() {
            Some(mut recording) => match recording.finish() {
                Ok(Some(data)) => {
                    eprintln!("Recording finished: {:?}", data.len());
                    let text = state
                        .runtime
                        .block_on(transcriber.transcribe(&state.config, data))
                        .unwrap();
                    eprintln!("Transcribed: {:?}", text);
                }
                Ok(None) => {
                    warn!("Recording finished but no data was recorded");
                }
                Err(e) => {
                    error!(error = ?e, "Failed to finish recording");
                }
            },
            None => match state.recorder.start_recording() {
                Ok(handle) => {
                    state.active_recording = Some(handle);
                }
                Err(e) => {
                    error!(error = ?e, "Failed to start recording");
                }
            },
        }
    }
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
