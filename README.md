# Whisp

A lightweight desktop speech-to-text tool that supports both OpenAI's
transcription API and local Whisper models. Whisp provides a simple interface
for converting speech to text with minimal resource overhead.

[![Crates.io][crates-badge]][crates-url] [![MIT licensed][mit-badge]][mit-url]

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

### Recommended: Local Whisper (No API Key Required)

The best experience is with local Whisper models - everything runs on your
machine with no API key or internet connection needed. Just install and go:

```sh
cargo install whisp --features local-whisper
whisp
```

On first launch, whisp will automatically download the default model
(large-v3-turbo-q8_0, ~834 MiB) and start working immediately.

**Requirements:** CMake must be installed (for building whisper.cpp). On Linux,
ALSA, GTK3, and X11 libraries are also required - a `shell.nix` is provided for
Nix users.

On macOS, Metal GPU acceleration is used automatically, with CoreML support for
~3x faster encoding via the Apple Neural Engine.

### Alternative: OpenAI API

If you prefer cloud transcription or can't build local whisper:

```sh
cargo install whisp
```

This requires an OpenAI API key configured in `whisp.toml`.

## Configuration

Configuration is managed through a `whisp.toml` file located in your system's
configuration directory. The whisp tray menu has an option to copy the
configuration file path to the clipboard.

### Example: Local Whisper Backend

When installed with `--features local-whisper`, whisp works with zero
configuration. Optionally customize with:

```toml
local_model = "large-v3-turbo-q8_0"
```

On first launch, the model is automatically downloaded (~834 MiB for
large-v3-turbo-q8_0). The large-v3-turbo model offers excellent transcription
quality and runs fast on modern hardware, especially MacBooks with Metal/CoreML
acceleration.

**Quick start:** For faster initial setup, use a smaller model like
`tiny.en-q8_0` (~42 MiB) or `base.en-q8_0` (~78 MiB).

### Example: OpenAI Backend

```toml
openai_key = "your-api-key"
hotkey = "shift+super+Semicolon"
language = "en"
model = "gpt-4o-mini-transcribe"
restore_clipboard = true
auto_paste = true
```

### Configuration Options

| Option              | Default                  | Description                                    |
| ------------------- | ------------------------ | ---------------------------------------------- |
| `backend`           | (depends on build)       | Transcription backend: `openai` or `local`     |
| `hotkey`            | `shift+super+Semicolon`  | Global hotkey to trigger recording             |
| `openai_key`        | (required for openai)    | Your OpenAI API key                            |
| `local_model`       | `large-v3-turbo-q8_0`    | Local Whisper model (see table below)          |
| `coreml`            | `true`                   | Enable CoreML acceleration (macOS only)        |
| `language`          | (none)                   | Language hint for transcription (e.g., "en")   |
| `model`             | `gpt-4o-mini-transcribe` | OpenAI transcription model                     |
| `restore_clipboard` | `false`                  | Restore clipboard contents after pasting       |
| `auto_paste`        | `true`                   | Automatically paste transcription              |
| `discard_duration`  | `0.5`                    | Discard recordings shorter than this (seconds) |
| `retries`           | `5`                      | Number of retries on API failure               |

The `backend` default is `local` when built with `--features local-whisper`,
otherwise `openai`.

### Available Local Models

Models are downloaded from [ggerganov/whisper.cpp on
HuggingFace](https://huggingface.co/ggerganov/whisper.cpp). Model names must
match exactly as shown below.

| Model                 | Size    | Notes              |
| --------------------- | ------- | ------------------ |
| `tiny`                | 75 MiB  | Fastest            |
| `tiny-q5_1`           | 31 MiB  |                    |
| `tiny-q8_0`           | 42 MiB  |                    |
| `tiny.en`             | 75 MiB  | English-only       |
| `tiny.en-q5_1`        | 31 MiB  |                    |
| `tiny.en-q8_0`        | 42 MiB  |                    |
| `base`                | 142 MiB |                    |
| `base-q5_1`           | 57 MiB  |                    |
| `base-q8_0`           | 78 MiB  |                    |
| `base.en`             | 142 MiB | English-only       |
| `base.en-q5_1`        | 57 MiB  |                    |
| `base.en-q8_0`        | 78 MiB  | English-only       |
| `small`               | 466 MiB |                    |
| `small-q5_1`          | 181 MiB |                    |
| `small-q8_0`          | 252 MiB |                    |
| `small.en`            | 466 MiB | English-only       |
| `small.en-q5_1`       | 181 MiB |                    |
| `small.en-q8_0`       | 252 MiB |                    |
| `small.en-tdrz`       | 465 MiB | Tinydiarize        |
| `medium`              | 1.5 GiB |                    |
| `medium-q5_0`         | 514 MiB |                    |
| `medium-q8_0`         | 785 MiB |                    |
| `medium.en`           | 1.5 GiB | English-only       |
| `medium.en-q5_0`      | 514 MiB |                    |
| `medium.en-q8_0`      | 785 MiB |                    |
| `large-v1`            | 2.9 GiB |                    |
| `large-v2`            | 2.9 GiB |                    |
| `large-v2-q5_0`       | 1.1 GiB |                    |
| `large-v2-q8_0`       | 1.5 GiB |                    |
| `large-v3`            | 2.9 GiB |                    |
| `large-v3-q5_0`       | 1.1 GiB |                    |
| `large-v3-turbo`      | 1.5 GiB | Best speed/quality |
| `large-v3-turbo-q5_0` | 547 MiB |                    |
| `large-v3-turbo-q8_0` | 834 MiB | **Default**        |

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

## CoreML Acceleration (macOS)

On macOS, the local backend automatically uses CoreML to run the encoder on
Apple's Neural Engine, providing ~3x faster encoding performance. This is
enabled by default.

On first use, whisp downloads a pre-built CoreML encoder model from HuggingFace
(15-1200 MiB depending on model size). The first transcription after launch may
be slow as macOS compiles the model for the Neural Engine, but subsequent
transcriptions are fast. Since whisp is designed to run in the background
indefinitely, this one-time warmup cost is negligible in practice.

CoreML encoders work with all GGML models, including quantized variants. The
encoder runs on the Neural Engine while the decoder uses the GGML model on
CPU/Metal.

To disable CoreML and use Metal-only acceleration:

```toml
coreml = false
```

## License

Whisp is licensed under the [MIT
license](https://github.com/cgbur/whisp/blob/main/LICENSE).
