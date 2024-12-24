import yt_dlp

class YTDownloader:
    def __init__(self, path, filename, audio_only=False):
        self.path = path
        self.filename = filename
        self.audio_only = audio_only

    def download(self, url):
        ydl_opts = {
            'outtmpl': f'{self.path}/{self.filename}.%(ext)s',
            'writethumbnail': True,
            'writeinfojson': True,
            'writesubtitles': True,
            'subtitleslangs': ['all'],
            'postprocessors': [{
                'key': 'FFmpegMetadata'
            }]
        }

        if self.audio_only:
            ydl_opts['format'] = 'bestaudio/best'
            ydl_opts['postprocessors'].append({
                'key': 'FFmpegExtractAudio',
                'preferredcodec': 'mp3',
                'preferredquality': '192',
            })
            ydl_opts['postprocessors'].append({
                'key': 'EmbedThumbnail'
            })

        else:
            ydl_opts['format'] = 'bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]'
            ydl_opts['postprocessors'].append({
                'key': 'FFmpegEmbedThumbnail'
            })

        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            ydl.download([url])

def telecharger_lrc(nom_musique, path):
    nom_fichier = nom_musique.replace('/', '-')
    url = f"https://www.lrclib.net/api/search?q={nom_musique}"
    response = requests.get(url, verify=False)
    if response.status_code == 200:
        try:
            data = response.json()
            if isinstance(data, list) and len(data) > 0:
                synced_lyrics = data[0].get('syncedLyrics', '')
                if synced_lyrics:
                    with open(f"{path}{nom_fichier}.lrc", 'w', encoding='utf-8') as f:
                        f.write(synced_lyrics)
                    print(f"Les paroles synchronisées ont été écrites dans {nom_fichier}.lrc")
        except ValueError:
            print("Erreur lors de la conversion de la réponse en JSON.")
    else:
        print(f"Erreur HTTP: {response.status_code}")