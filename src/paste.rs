use enigo::Direction::{Click, Press, Release};
use enigo::{Enigo, Key, Keyboard};

fn paste(enigo: &mut Enigo) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    let paste_key = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let paste_key = Key::Control;

    enigo.key(paste_key, Press)?;
    enigo.key(Key::Unicode('v'), Click)?;
    enigo.key(paste_key, Release)?;

    Ok(())
}

/// Spawns a paste task that listens for paste envents. The sender returned
/// is used to trigger these events. Enigo is not Send, so we keep it
/// parked in a nice little thread and talk to it via the sender.
pub fn spawn_paste_task() -> std::sync::mpsc::Sender<()> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
        while receiver.recv().is_ok() {
            paste(&mut enigo).unwrap();
        }
    });
    sender
}
