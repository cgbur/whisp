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

- **CMake** must be installed (for building whisper.cpp)

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
local_model = "base.en-q8_0"
hotkey = "shift+super+Semicolon"
language = "en"
```

On first launch with local backend, the model will be automatically downloaded
(~78 MiB for base.en-q8_0).

**Note:** The default model (`base.en-q8_0`) is English-only. If you need
multilingual support, use a non-English model like `base-q8_0` or `small-q8_0`.

### Configuration Options

| Option              | Default                  | Description                                          |
| ------------------- | ------------------------ | ---------------------------------------------------- |
| `backend`           | `openai`                 | Transcription backend: `openai` or `local`           |
| `hotkey`            | `shift+super+Semicolon`  | Global hotkey to trigger recording                   |
| `openai_key`        | (required for openai)    | Your OpenAI API key                                  |
| `local_model`       | `base.en-q8_0`           | Local Whisper model (see table below)                |
| `language`          | (none)                   | Language hint for transcription (e.g., "en")         |
| `model`             | `gpt-4o-mini-transcribe` | OpenAI transcription model                           |
| `restore_clipboard` | `false`                  | Restore clipboard contents after pasting             |
| `auto_paste`        | `true`                   | Automatically paste transcription                    |
| `discard_duration`  | `0.5`                    | Discard recordings shorter than this (seconds)       |
| `retries`           | `5`                      | Number of retries on API failure                     |

### Available Local Models

Models are downloaded from [ggerganov/whisper.cpp on HuggingFace](https://huggingface.co/ggerganov/whisper.cpp).
Model names must match exactly as shown below.

| Model | Size | Notes |
| --- | --- | --- |
| `tiny` | 75 MiB | Fastest |
| `tiny-q5_1` | 31 MiB | |
| `tiny-q8_0` | 42 MiB | |
| `tiny.en` | 75 MiB | English-only |
| `tiny.en-q5_1` | 31 MiB | |
| `tiny.en-q8_0` | 42 MiB | |
| `base` | 142 MiB | |
| `base-q5_1` | 57 MiB | |
| `base-q8_0` | 78 MiB | |
| `base.en` | 142 MiB | English-only |
| `base.en-q5_1` | 57 MiB | |
| `base.en-q8_0` | 78 MiB | **Default**, English-only |
| `small` | 466 MiB | |
| `small-q5_1` | 181 MiB | |
| `small-q8_0` | 252 MiB | |
| `small.en` | 466 MiB | English-only |
| `small.en-q5_1` | 181 MiB | |
| `small.en-q8_0` | 252 MiB | |
| `small.en-tdrz` | 465 MiB | Tinydiarize |
| `medium` | 1.5 GiB | |
| `medium-q5_0` | 514 MiB | |
| `medium-q8_0` | 785 MiB | |
| `medium.en` | 1.5 GiB | English-only |
| `medium.en-q5_0` | 514 MiB | |
| `medium.en-q8_0` | 785 MiB | |
| `large-v1` | 2.9 GiB | |
| `large-v2` | 2.9 GiB | |
| `large-v2-q5_0` | 1.1 GiB | |
| `large-v2-q8_0` | 1.5 GiB | |
| `large-v3` | 2.9 GiB | |
| `large-v3-q5_0` | 1.1 GiB | |
| `large-v3-turbo` | 1.5 GiB | Best speed/quality |
| `large-v3-turbo-q5_0` | 547 MiB | |
| `large-v3-turbo-q8_0` | 834 MiB | |

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
