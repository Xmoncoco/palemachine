# palemachine
A YouTube downloader for server with a Spotify picture finder.

## Why the name palemachine?
Because I love the album "palemachine" by bo-en.

## How to use it?

Firstly, you will need:
- Google YouTube Data API key
- Spotify API client ID and secret
- ffmpeg and python3 in your PATH

> **Note:** Your credentials will be used for showing thumbnails for music.

> **Information:** You can run the project on Windows, but you will need to "translate" the `start.sh` script. However, I can't guarantee it will work 100% as I don't have a Windows computer, so Windows compatibility is not planned.

### Steps to use:

1. Clone the repository:
   ```bash
   git clone https://github.com/Xmoncoco/palemachine.git
   ```

2. Navigate to the project directory
    ```bash
    cd palemachine
    ```

3. Create a .env file in the project directory and add your credentials
    ```.env
    YOUTUBE_API_KEY=your_youtube_api_key
    SPOTIFY_CLIENT_ID=your_spotify_client_id
    SPOTIFY_CLIENT_SECRET=your_spotify_client_secret
    ````

4. Ensure ffmpeg is installed and available in your PATH.
    if you type ffmpeg in your terminal and get a message __"Use -h to get full help or, even better, run 'man ffmpeg'"__ you are good.

5. now cross your fingers and run ./start.sh :
    ```bash 
    ./start.sh
    ```
## Troubleshooting
If you encounter any issues, make sure to check the following:

- Ensure all dependencies are installed correctly.
- Verify that your .env file contains the correct credentials.
- Check that ffmpeg is properly installed and accessible from your PATH.

and if nothing work make a issue I'll try to fix it.

## Contributing
If you would like to contribute to this project, please fork the repository and submit a pull request. We welcome all contributions!

## License
This project is licensed under the MIT License. See the LICENSE file for more details.

