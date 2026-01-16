# Development

## Formatting

Use nightly rustfmt for full formatting support:

```bash
cargo +nightly fmt
```

## Linting

Always run clippy with tests:

```bash
cargo clippy --tests
```

## Building with local-whisper feature

On Linux, use nix-shell to get all required dependencies (GTK, ALSA, etc.):

```bash
nix-shell --run "cargo build --features local-whisper"
```

Or enter the shell interactively:

```bash
nix-shell
cargo build --features local-whisper
cargo clippy --tests --features local-whisper
```

The shell.nix fetches nixpkgs-unstable which has Rust 1.91+ (required for edition 2024).

## Building without local-whisper

Standard build without local whisper (uses OpenAI API):

```bash
cargo build
```
