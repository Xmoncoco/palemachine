from django.shortcuts import render
from django.http import HttpResponse
from .Downloader import YTDownloader
import uuid 
import os
import requests 

basepath = '/Users/milhanhadjadji/dev/django/palemachine/palemachine/'
YOUTUBE_API_KEY = os.getenv('YOUTUBE_API_KEY')
SPOTIFY_CLIENT_ID = os.getenv('SPOTIFY_CLIENT_ID')
SPOTIFY_CLIENT_SECRET = os.getenv('SPOTIFY_CLIENT_SECRET')

def download_form(request):
    return render(request, f'download_form.html')

def instantdl(request):
    url = request.GET.get('yt_url')
    filename = request.GET.get('filename')
    uuid2 = uuid.uuid4()
    download_path = os.path.join(basepath, str(uuid2))
    os.mkdir(download_path)
    downloader = YTDownloader(download_path, filename, audio_only=True)
    downloader.download(url)
    return HttpResponse(f"Download complete. Files are saved in directory: {uuid2}")


def manualdl(request):
    url = request.GET.get('yt_url')
    filename = request.GET.get('filename')
    img_url = request.GET.get('img_url')
    
    uuid2 = uuid.uuid4()
    download_path = os.path.join(basepath, str(uuid2))
    os.mkdir(download_path)
    
    # Download the thumbnail image
    img_data = requests.get(img_url).content
    thumbnail_path = os.path.join(download_path, 'thumbnail.jpg')
    with open(thumbnail_path, 'wb') as handler:
        handler.write(img_data)
    
    downloader = YTDownloader(download_path, filename, audio_only=True)
    downloader.download(url)
    video_path = os.path.join(download_path, f'{filename}.mp3')
    os.run(f'ffmpeg -i {video_path} -i {thumbnail_path} -map 0 -map 1 -c copy -c:v:1 png -disposition:1 attached_pic {filename}_thumbnail.mp3')
    
    return HttpResponse(f"Download complete. Files are saved in directory: {uuid2}")
    
def imgquestion(request):
    uuid2 = uuid.uuid4()
    url = request.GET.get('yt_url')
    friendlyname = request.GET.get('friendlyname')
    
    # Récupération du titre via l'API YouTube
    video_id = url.split("v=")[-1].split("&")[0]
    youtube_api_url = f"https://www.googleapis.com/youtube/v3/videos?part=snippet&id={video_id}&key={YOUTUBE_API_KEY}"
    youtube_response = requests.get(youtube_api_url).json()
    title = youtube_response['items'][0]['snippet']['title']
    print(title)
    
    # Recherche Spotify avec le titre et le friendlyname
    auth_response = requests.post(
        'https://accounts.spotify.com/api/token',
        data={'grant_type': 'client_credentials'},
        auth=(SPOTIFY_CLIENT_ID, SPOTIFY_CLIENT_SECRET)
    )
    access_token = auth_response.json().get('access_token')
    
    track_images = []
    search_queries = [friendlyname,title]
    headers = {'Authorization': f'Bearer {access_token}'}
    
    for search in search_queries:
        spotify_api_url = f"https://api.spotify.com/v1/search?q={search}&type=track"
        spotify_response = requests.get(spotify_api_url, headers=headers).json()
        tracks = spotify_response.get('tracks', {}).get('items', [])
        track_images.extend([track['album']['images'][0]['url'] for track in tracks if track.get('album') and track['album'].get('images')])
        print(track_images)

    return render(
        request,
        'manualdl_result.html',
        {
            'uuid2': uuid2,
            'title': title,
            'track_images': track_images,
            'friendlyname': friendlyname
        }
    )