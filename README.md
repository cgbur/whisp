# Whisp

A lightweight desktop speech-to-text tool powered by modern models like
[OpenAI's Whisper](https://github.com/openai/whisper). Whisp provides a simple
interface for converting speech to text with minimal resource overhead.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/whisp.svg
[crates-url]: https://crates.io/crates/whisp
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/cgbur/whisp/blob/main/LICENSE

## Overview

Whisp offers an unobtrusive and customizable way to transcribe your voice into
text. It operates as a globally available desktop application. Activate it via a
hotkey, and it can automatically paste the transcribed text into any focused
input field.

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

Configuration is managed through a `whisp.toml` file located in your systems
configuration directory. The whisp drop-down has an option to copy the
configuration file path to the clipboard.

```toml
hotkey = "shift+super+Semicolon"
openai_key = "your-api-key"
language = "en"
model = "whisper-1"
restore_clipboard = true
auto_paste = false
```

## Usage

To start using Whisp, define your preferred hotkey, configure the model, and run
the application. You can then trigger voice recording via the hotkey and receive
transcriptions automatically.

### Common Use Cases

- **Messaging**: Quickly respond to messages in chat applications like Discord
  or Slack.

- **Document Writing**: Speak freely to draft large amounts of text quickly.
  Then apply post-This is the test.This is the test.I think this is a very good and formal test. yourself or with the help of a language model to
  refine the text.

- **Code Commenting**: Dictate comments directly into your editor. Note this
  tool does not write code well. However, perhaps this can change when the
  automatic post processing is added. Reach out if you are interested in
  contributing.

## License

Whisp is licensed under the [MIT
license](https://github.com/cgbur/whisp/blob/main/LICENSE).
