use enigo::Direction::{Click, Press, Release};
use enigo::{Enigo, Key, Keyboard};

pub fn paste(enigo: &mut Enigo) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    let paste_key = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let paste_key = Key::Control;

    enigo.key(paste_key, Press)?;
    enigo.key(Key::Unicode('v'), Click)?;
    enigo.key(paste_key, Release)?;

    Ok(())
}
