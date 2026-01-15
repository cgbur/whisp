//! Test binary for transcription.
//!
//! Usage: transcribe-test <audio_file> <api_key> [model]

use std::env;
use std::fs;
use std::time::Instant;

use whisp_transcribe::{OpenAIClient, OpenAIConfig, Transcriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <audio_file> <api_key> [model]", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} test.wav sk-... gpt-4o-mini-transcribe", args[0]);
        std::process::exit(1);
    }

    let audio_file = &args[1];
    let api_key = &args[2];
    let model = args.get(3).map(|s| s.as_str());

    // Read audio file
    println!("Reading audio file: {}", audio_file);
    let audio = fs::read(audio_file)?;
    println!(
        "Audio size: {} bytes ({:.2} KB)",
        audio.len(),
        audio.len() as f64 / 1024.0
    );

    // Configure client
    let mut config = OpenAIConfig::new(api_key);
    if let Some(model) = model {
        config = config.with_model(model);
    }
    println!("Using model: {}", config.model());

    let client = OpenAIClient::new(config);

    // Send transcription request
    println!("Sending transcription request...");
    let start = Instant::now();

    let text = client.transcribe(&audio, None).await?;
    let elapsed = start.elapsed();

    println!();
    println!("Transcription completed in {:.2}s", elapsed.as_secs_f64());
    println!("---");
    println!("{}", text);
    println!("---");

    Ok(())
}
