# ADR-006: Audio Output on WSL2 (Development Environment)

**Date**: 2026-05-14
**Status**: Accepted — Confirmed Working

## Context

Development happens on Arch Linux running inside WSL2 on Windows. WSL2 traditionally has no audio support — it lacks `/dev/snd` devices. This must be confirmed working before committing to the Rust/cpal stack.

## Finding

**WSLg (Windows Subsystem for Linux GUI)** ships a PulseAudio server at `unix:/mnt/wslg/PulseServer` that bridges to the Windows audio system via an RDP sink. This is available automatically on Windows 11 / WSL2 with WSLg enabled.

Verified on 2026-05-14:
```
$ pactl info
Server String: unix:/mnt/wslg/PulseServer
Default Sink: RDPSink
Default Sample Specification: s16le 2ch 44100Hz

$ python3 -c "... generate 440Hz WAV ..." | paplay  # exit code 0 ✓
```

**`cpal`** on Linux uses ALSA as its primary backend, which talks to PulseAudio via the ALSA-PulseAudio bridge (`libasound2-plugins`), or directly via the PulseAudio ALSA interface. Alternatively, cpal supports a PulseAudio backend directly (`features = ["jack"]` or the default ALSA path picks up the PA ALSA plugin).

## Decision

Use `cpal` with the default ALSA backend. ALSA must be configured to route through the WSLg PulseAudio server via the `alsa-plugins` bridge.

**One-time setup required on Arch Linux WSL2:**

```bash
# 1. Install the PulseAudio ALSA bridge plugin
sudo pacman -S alsa-plugins

# 2. Configure ALSA to use PulseAudio as the default output
cat >> ~/.asoundrc << 'EOF'
pcm.!default {
    type pulse
    fallback "sysdefault"
}
ctl.!default {
    type pulse
    fallback "sysdefault"
}
EOF
```

The `PULSE_SERVER` environment variable is auto-set by WSLg:
```
PULSE_SERVER=unix:/mnt/wslg/PulseServer
```

**Why this is needed**: WSL2 has no ALSA hardware devices (`/proc/asound/cards` is empty). Without `alsa-plugins` and the `~/.asoundrc` config, cpal's `default_output_device()` finds nothing and returns "device not available". With the plugin, ALSA's default PCM device becomes a virtual device backed by PulseAudio.

**Confirmed broken**: `cargo run` without setup → `Error: The requested device is no longer available.`
**Confirmed working**: After setup, `cargo run` → audio plays.

## Production / Cross-Platform

On non-WSL targets, `cpal` uses the platform-native audio:
- **Linux**: ALSA (default) or JACK
- **macOS**: CoreAudio
- **Windows**: WASAPI (default) or DirectSound

No conditional compilation or platform-specific code needed in the application.

## Consequences

- Development on WSL2 + WSLg works with no special configuration
- The audio path in development (WSLg PulseAudio → Windows WASAPI → speakers) has ~10-30ms additional latency vs. native Linux — acceptable for a musical instrument application (real musicians tolerate up to ~10ms; CLI instrument is more forgiving)
- CI (if ever configured on Linux runners) needs a virtual audio device (e.g., `pulseaudio --start` with a null sink) to run audio integration tests
