# Whisp

A lightweight desktop speech-to-text tool powered by OpenAI's transcription API.
Whisp provides a simple interface for converting speech to text with minimal
resource overhead.

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

Currently the only way to install this is via cargo:

```sh
cargo install whisp
whisp
# if cargo bin is not in your path
~/.cargo/bin/whisp
```

## Configuration

Configuration is managed through a `whisp.toml` file located in your system's
configuration directory. The whisp tray menu has an option to copy the
configuration file path to the clipboard.

```toml
hotkey = "shift+super+Semicolon"
openai_key = "your-api-key"
language = "en"
model = "gpt-4o-mini-transcribe"
restore_clipboard = true
auto_paste = true
discard_duration = 0.5
retries = 5
```

### Configuration Options

| Option              | Default                  | Description                                    |
| ------------------- | ------------------------ | ---------------------------------------------- |
| `hotkey`            | `shift+super+Semicolon`  | Global hotkey to trigger recording             |
| `openai_key`        | (required)               | Your OpenAI API key                            |
| `language`          | (none)                   | Language hint for transcription (e.g., "en")   |
| `model`             | `gpt-4o-mini-transcribe` | Transcription model to use                     |
| `restore_clipboard` | `false`                  | Restore clipboard contents after pasting       |
| `auto_paste`        | `true`                   | Automatically paste transcription              |
| `discard_duration`  | `0.5`                    | Discard recordings shorter than this (seconds) |
| `retries`           | `5`                      | Number of retries on API failure               |

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

## License

Whisp is licensed under the [MIT
license](https://github.com/cgbur/whisp/blob/main/LICENSE).
