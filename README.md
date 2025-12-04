# Palemachine

Palemachine is a robust web-based music downloader and manager built with Rust (Actix-web). It bridges the gap between YouTube and Spotify, allowing you to download high-quality audio from YouTube videos or playlists while fetching accurate metadata and cover art from Spotify.

## Features

-   **YouTube Downloading**: Download individual videos or entire playlists.
-   **Spotify Integration**: Automatically fetches high-quality album art and metadata from Spotify.
-   **WebAuthn Authentication**: Secure, passwordless login using Passkeys (with a fallback password).
-   **Responsive Web UI**: Clean interface for managing downloads and settings.
-   **Configurable**: Easy-to-adjust settings via `config.toml`.

## Prerequisites

Before you begin, ensure you have the following installed:

-   [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
-   [Python 3](https://www.python.org/downloads/) (required for the underlying download scripts)
-   `ffmpeg` (usually required for audio processing)

## Installation

### Arch Linux (AUR)

You can install Palemachine directly from the AUR using your preferred AUR helper (e.g., `yay` or `paru`):

```bash
yay -S palemachine
```

### Manual Installation

#### Option A: From Release

1.  Download the latest release from the [Releases page](https://github.com/Xmoncoco/palemachine/releases).
2.  Extract the archive.
3.  Configure your `.env` file (see Configuration section).
4.  Run the executable: `./palemachine`.

#### Option B: Build from Source

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Xmoncoco/palemachine.git
    cd palemachine
    ```

2.  **Build the project:**
    Use the provided build script to compile the project and set up the environment (including Python dependencies).
    ```bash
    ./installation_scripts/build.sh
    ```
    This will create a `package` directory containing everything you need.

3.  **Run:**
    Navigate to the package directory and start the server.
    ```bash
    cd package
    ./palemachine
    ```

## Configuration

### Environment Variables (.env)

The application requires API keys to function. A `.env` file is generated in your installation directory (or `package` folder). Edit it to add your keys:

```bash
nano .env
```

-   `YOUTUBE_API_KEY`: Get this from the [Google Cloud Console](https://console.cloud.google.com/).
-   `SPOTIFY_CLIENT` & `SPOTIFY_SECRET`: Get these from the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard/).

### Application Settings (config.toml)

The `config.toml` file controls server settings. It is automatically created on first run or by the build script.

```toml
path = "./downloads"      # Directory to save downloads
port = 9999               # Web server port
password = "admin"        # Fallback password
domain = "localhost"      # Domain for WebAuthn (e.g., localhost or your-domain.com)
```

## Usage

1.  Open your browser and navigate to `http://localhost:9999` (or the port you configured).
2.  Log in using the default password (`admin`) or register a Passkey for future logins.
3.  Use the interface to submit YouTube URLs for downloading.
4.  The server will process the request, download the audio, and tag it with metadata found via Spotify.

## Development

To run the project in development mode with hot-reloading (if configured) or standard debug build:

```bash
cargo run
```

## License

[MIT](LICENSE)
