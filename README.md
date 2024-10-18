# Whisp

Whisp is a lightweight desktop speech-to-text tool designed for simplicity and
efficiency. Modern speech-to-text models like [OpenAI's
Whisper](https://github.com/openai/whisper) offer impressive accuracy and
represent a significant improvement in usability over older technologies. Whisp
aims to provide a minimal, unobtrusive interface to these models, allowing
speech-to-text input on anything you can type in.

### Goals

1. **Robust**: Broken tools are not useful. Whisp is stable and reliable. Errors
   are handled gracefully, and retries are automatic when possible.

2. **Minimal**: Resource intensive tools with many features and poor execution
   are not useful. Whisp is lightweight and unobtrusive. It will do this one
   thing, and do it well.

3. **Secure**: Handling API keys, voice data, and registering a global
   hotkey to enable recordings is a lot of power to give to a randomly
   downloaded tool. Whisp is open source, and designed to be simple and readble.

## Status

It is working, but bare bones. Using day to day to gather feedback and improve
in free time. Currently it only supports OpenAI's api.

Basic configuration in `whisp.toml`:

Hotkeys are really bad right now, need to make custom hotkey parser.

```toml
hotkey = "shift+super+Semicolon"
openai_key = "not-a-real-key"
language = "en"
model = "whisper-1"
restore_clipboard = true
auto_paste = false
```
