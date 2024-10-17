# Whisp

Whisp is a lightweight desktop speech-to-text tool designed for simplicity and
efficiency. Modern speech-to-text models like [OpenAI's
Whisper](https://github.com/openai/whisper) offer impressive accuracy and
represent a significant improvement in usability over older technologies. Whisp
aims to provide a minimal, unobtrusive interface to these models, allowing
seamless speech-to-text functionality across all your desktop applications.

### Goals

These are aspirational, as no code has been written yet at the time of this README.

1. **Robust**: Broken tools are not useful. Whisp is stable and reliable. Errors
   are handled gracefully, and retries are automatic when possible.

2. **Minimal**: Resource intensive tools with many features and poor execution
   are not useful. Whisp will be lightweight and unobtrusive. It will do this one
   thing, and do it well.

3. **Secure**: Handling API keys, voice data, and registering a global
   hotkey to enable recordings are not only security concerns but also require a
   lot of trust in a tool from the internet. Whisp handles your data with best
   practices, but in terms of trust, the best you can do is review the source
   and build it yourself.

## Status

It is working, but bare bones. Using day to day to gather feedback and improve
in free time.

### TODO

- Add indication when mic data starts being recieved. Currently difficult to know when
  exactly you can start speaking. Would like to have this indication be a visual
  in the bar.
- Notifications on errors.
- Basic settings page
- Document settings
- Add post processing
- Add audio pre-filter to remove dead air (saves money, smaller uploads etc.)
- Very difficult to test, need to figure out how to do this.
- Set up basic CI/CD, releases, etc.
