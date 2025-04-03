# rsensor

A lightweight system monitoring tool written in Rust, inspired by psensor.

![screenshot](assets/screenshots/screenshot.png)
## Features

- Real-time CPU monitoring (usage, temperature, per-core stats)
- Memory usage statistics with min/max tracking
- GPU monitoring for both NVIDIA and AMD GPUs
- Terminal-based user interface with clean, responsive layout
- Low system resource usage

## Installation

### Arch Linux

You can install rsensor using the provided PKGBUILD:

```bash
# Clone the repository
git clone https://github.com/tahuffman1s/rsensor.git
cd rsensor

# Build and install the package
makepkg -si