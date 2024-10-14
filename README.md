# Whisp

Whisp is a robust, minimal, and secure speech-to-text application that runs in
the background, allowing users to dictate text into any application that accepts
keyboard input. Designed to be lightweight and unobtrusive, Whisp aims to
seamlessly integrate into the user's workflow.

**Goals**

These are aspirational, as not a single line of code has been written at the
time of this README.

1. **Robust**: Tools should be dependable and work as expected. Downtime in
   services should have fallbacks. Whisp will gracefully handle errors and retry
   when possible. Downtime in a tool like this is extremely frustrating and can be
   especially disruptive to users who rely on it. _It aspires to be relied upon._

2. **Minimal**: Whisp should be lightweight and unobtrusive. When not in use, it
   should not activate the microphone or consume system resources. When in use, it
   should use as few resources as possible. Whisp will focus on providing a clean
   desktop experience without unnecessary features, UI, or configuration.

3. **Secure**: By its very nature, this tool will handle users' voice data and
   may, if configured, send it to a third-party service for transcription. Users
   are installing an application they did not necessarily write and are trusting it
   with their data. Minimal dependencies, open source, and easy-to-audit code are
   all important to ensure that users can trust the application.
