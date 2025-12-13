# Soundcheck

A terminal-based audio monitoring application that displays real-time audio levels from selected input channels and exits when sound exceeds a specified threshold.

Ispired by the SP404 feature, the idea for this program came from a tool to make one shot sample recording easier for the solo music producer. Chain this with any audio recoridng program, such as ffmpg, and the recording will hold until the dB threshold is reached, the start recording immediately. This will result in a clean sample with no need to trim empty space at the beginning.
## Installation 🚀

### From Crates.io

```bash
cargo install soundcheck
```

### From Homebrew

```bash
brew tap isfopo/homebrew-tap

brew install soundcheck
```

<!-- ### From Scoop (Windows) -->
<!---->
<!-- ```bash -->
<!-- scoop bucket add username-scoop-bucket https://github.com/username/soundcheck-scoop-bucket -->
<!---->
<!-- scoop install soundcheck -->
<!-- ``` -->
<!---->

### From Chocolatey (Windows)

```bash
choco install soundcheck
```

<!-- ### From AUR (Arch Linux) -->
<!---->
<!-- ```bash -->
<!-- # Using yay -->
<!-- yay -S soundcheck -->
<!-- ``` -->
<!---->
<!-- ### From Debian/Ubuntu Packages -->
<!---->
<!-- ```bash -->
<!-- # Download .deb from releases -->
<!-- sudo dpkg -i soundcheck_*.deb -->
<!-- sudo apt install -f  # Install dependencies if needed -->
<!-- ``` -->
<!---->
<!-- ### AppImage (Universal Linux) -->
<!---->
<!-- ```bash -->
<!-- # Download AppImage from releases -->
<!-- chmod +x soundcheck-*.AppImage -->
<!-- ./soundcheck-*.AppImage --help -->
<!-- ``` -->
<!---->

### From Source

```bash
git clone <repository-url>
cd soundcheck
cargo build --release
# Binary will be at target/release/soundcheck
```

## Contributing & Development 🤝

### Release Process

For detailed information about creating and publishing releases, see [RELEASE.md](RELEASE.md).

## Usage 📖

### Basic Usage

```bash
# List available audio devices (interactive selection)
soundcheck list

# Pipe device selection to detect
soundcheck list | xargs soundcheck detect --device

# Monitor max levels for 5 seconds
soundcheck max --seconds 5 --channels 0,1

# Monitor until Enter is pressed
soundcheck max --channels 0,1

# Calculate average levels over 5 seconds
soundcheck average --seconds 5 --channels 0,1

# Calculate average until Enter is pressed
soundcheck average --channels 0,1

# Get quiet output for scripting
soundcheck max --seconds 5 --quiet
```

### Commands

- `detect`: Monitor audio levels and exit when threshold is exceeded
- `list`: List available audio input devices
- `max`: Monitor audio and report maximum levels detected
- `average`: Monitor audio and report average levels detected

### Detect Command Options

| Option        | Description                                    | Default        | Example                        |
| ------------- | ---------------------------------------------- | -------------- | ------------------------------ |
| `--threshold` | Audio threshold in dB (-60 to 0)               | 0              | `--threshold -30`              |
| `--min-db`    | Minimum dB level for display (-100 to 0)       | -60            | `--min-db -80`                 |
| `--channels`  | Audio channels to monitor (comma-separated)    | [0]            | `--channels 0,1`               |
| `--device`    | Audio input device name                        | Default device | `--device "USB Microphone"`    |

### Max Command Options

| Option        | Description                                    | Default        | Example                        |
| ------------- | ---------------------------------------------- | -------------- | ------------------------------ |
| `--seconds`   | Monitoring duration in seconds                 | Until Enter     | `--seconds 10`                 |
| `--min-db`    | Minimum dB level for display (-100 to 0)       | -60            | `--min-db -80`                 |
| `--channels`  | Audio channels to monitor (comma-separated)    | [0]            | `--channels 0,1`               |
| `--device`    | Audio input device name                        | Default device | `--device "USB Microphone"`    |
| `--quiet`     | Output only integer values without labels      | false          | `--quiet`                      |

### Average Command Options

| Option        | Description                                    | Default        | Example                        |
| ------------- | ---------------------------------------------- | -------------- | ------------------------------ |
| `--seconds`   | Monitoring duration in seconds                 | Until Enter     | `--seconds 10`                 |
| `--min-db`    | Minimum dB level for display (-100 to 0)       | -60            | `--min-db -80`                 |
| `--channels`  | Audio channels to monitor (comma-separated)    | [0]            | `--channels 0,1`               |
| `--device`    | Audio input device name                        | Default device | `--device "USB Microphone"`    |
| `--quiet`     | Output only integer values without labels      | false          | `--quiet`                      |

### List Command

```bash
soundcheck list  # Interactive device selection (navigate with arrow keys, press Enter)
```

### Multi-Channel Monitoring

When monitoring multiple channels, the application displays separate gauges for each channel:

- **Single Channel**: Shows one gradient bar with dB labels
- **Multiple Channels**: Displays stacked gauges, one per channel
- **Threshold Detection**: Exits when ANY monitored channel exceeds the threshold

### Command Chaining Examples

```bash
# Continue to next command only if threshold reached on any channel
soundcheck detect --channels 0,1 && echo "Audio detected!"

# Run fallback command if user exits
soundcheck detect || echo "Monitoring cancelled by user"

# Error handling
soundcheck detect || echo "Failed to start monitoring"
```

## Requirements 📋

### System Requirements

- **macOS**: 10.15 or later
- **Linux**: Kernel 3.16+ with ALSA
- **Windows**: Windows 10+ with WASAPI

### Dependencies

- **Rust**: 1.70+ (for edition 2021)
- **Audio Libraries**: System audio frameworks
  - macOS: CoreAudio
  - Linux: ALSA
  - Windows: WASAPI

## Development 🛠️

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt
```

## Troubleshooting 🔍

### Common Issues

**"No default input device"**

- Ensure your microphone/audio input is connected and enabled
- Check system audio settings

**"Device not found"**

- Use `soundcheck list` to see available devices
- Use `soundcheck detect --help` to see available options
- Verify the device name spelling

**Blank screen on startup**

- Ensure terminal supports Unicode characters
- Try a different terminal emulator

**Audio levels not updating**

- Check that the correct audio device is selected
- Verify audio input permissions
- Ensure selected channels are valid for the device
- Test with different threshold values

### Debug Mode

```bash
# Run with verbose output
RUST_LOG=debug cargo run -- detect --threshold -20 --channels 0,1
```

## License 📄

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments 🙏

- [CPAL](https://github.com/RustAudio/cpal) - Cross-platform audio library
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Tokio](https://tokio.rs/) - Async runtime
- [Clap](https://github.com/clap-rs/clap) - Command line parsing

---
