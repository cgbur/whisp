# Whisp

A lightweight desktop speech-to-text tool that supports both OpenAI's
transcription API and local Whisper models. Whisp provides a simple interface
for converting speech to text with minimal resource overhead.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/whisp.svg
[crates-url]: https://crates.io/crates/whisp
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/cgbur/whisp/blob/main/LICENSE

## Overview

Whisp offers an unobtrusive and customizable way to transcribe your voice into
text. It runs in your system tray and is globally available. Press the hotkey to
start recording, speak as long as you want, then press the hotkey again to
transcribe and automatically paste into any focused input field.

Design principles:

- **Reliable**: Built to be stable and handle errors gracefully. Resilient in
  the face of errors. Retry and recovery.

- **Lightweight**: Resource-efficient, minimal system impact, simple.

## Installation

### Standard Installation (OpenAI API)

```sh
cargo install whisp
whisp
# if cargo bin is not in your path
~/.cargo/bin/whisp
```

### With Local Whisper Support

To use local Whisper models (no internet or API key required), build with the
`local-whisper` feature:

```sh
cargo install whisp --features local-whisper
```

**Requirements for local whisper:**

- **CMake** must be installed
- On **macOS Apple Silicon**, add the following to `~/.cargo/config.toml`:

```toml
[target.aarch64-apple-darwin]
rustflags = "-lc++ -l framework=Accelerate"
```

The local backend uses Metal GPU acceleration on Mac for fast transcription.

## Configuration

Configuration is managed through a `whisp.toml` file located in your system's
configuration directory. The whisp tray menu has an option to copy the
configuration file path to the clipboard.

### Example: OpenAI Backend

```toml
backend = "openai"
hotkey = "shift+super+Semicolon"
openai_key = "your-api-key"
language = "en"
model = "gpt-4o-mini-transcribe"
restore_clipboard = true
auto_paste = true
```

### Example: Local Whisper Backend

```toml
backend = "local"
local_model = "base-q8"
hotkey = "shift+super+Semicolon"
language = "en"
```

On first launch with local backend, the model will be automatically downloaded
(~82MB for base-q8).

### Configuration Options

| Option              | Default                  | Description                                          |
| ------------------- | ------------------------ | ---------------------------------------------------- |
| `backend`           | `openai`                 | Transcription backend: `openai` or `local`           |
| `hotkey`            | `shift+super+Semicolon`  | Global hotkey to trigger recording                   |
| `openai_key`        | (required for openai)    | Your OpenAI API key                                  |
| `local_model`       | `base-q8`                | Local Whisper model (see table below)                |
| `language`          | (none)                   | Language hint for transcription (e.g., "en")         |
| `model`             | `gpt-4o-mini-transcribe` | OpenAI transcription model                           |
| `restore_clipboard` | `false`                  | Restore clipboard contents after pasting             |
| `auto_paste`        | `true`                   | Automatically paste transcription                    |
| `discard_duration`  | `0.5`                    | Discard recordings shorter than this (seconds)       |
| `retries`           | `5`                      | Number of retries on API failure                     |

### Available Local Models

| Model               | Size    | Notes                        |
| ------------------- | ------- | ---------------------------- |
| `tiny-q8`           | ~44 MB  | Fastest, lowest quality      |
| `base-q8`           | ~82 MB  | Good default                 |
| `small-q8`          | ~264 MB | Better quality               |
| `medium-q8`         | ~823 MB | High quality                 |
| `large-v3-turbo-q5` | ~574 MB | Best speed/quality ratio     |

English-only variants (faster for English): `tiny-en`, `base-en`, `small-en`, `medium-en`

## Usage

1. Run `whisp` - it will appear in your system tray
2. Press the hotkey to start recording
3. Speak as long as you want
4. Press the hotkey again to transcribe and paste

### Common Use Cases

- **AI Coding Agents**: Voice dictate prompts to tools like Claude Code. Much
  faster than typing.

- **Messaging**: Quickly respond to messages in chat applications like Discord
  or Slack.

- **Document Writing**: Speak freely to draft large amounts of text quickly.

## Future Enhancements

- **CoreML Support**: The local backend could be enhanced to use CoreML for ~3x
  faster encoder performance on Mac via the Apple Neural Engine. This would
  require downloading additional encoder files.

## License

Whisp is licensed under the [MIT
license](https://github.com/cgbur/whisp/blob/main/LICENSE).
