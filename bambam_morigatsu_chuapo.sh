#!/usr/bin/env bash
#if you ask why the name check this music https://www.youtube.com/watch?v=Yeg--gomuv8 and becauso of the crash that I got when finishing the 
# Vérification des arguments
if [ -z "$1" ]; then
  echo "Usage: $0 <dossier>"
  exit 1
fi

dir="$1"

# Pour chaque fichier mp3 du dossier
for mp3 in "$dir"/*.mp3; do
  [ -e "$mp3" ] || continue  # Si aucun mp3, skip
  base="${mp3%.mp3}"
  jpg="${base}.jpg"
  # Si un jpg du même nom existe
  if [[ -f "$jpg" ]]; then
    echo "Ajout de la miniature $jpg à $mp3"
    ffmpeg -y -i "$mp3" -i "$jpg" -map 0 -map 1 -c copy -id3v2_version 3 \
      -metadata:s:v title="Album cover" -metadata:s:v comment="Cover (front)" \
      "${mp3}.tmp.mp3"
    if [[ $? -eq 0 ]]; then
      mv "${mp3}.tmp.mp3" "$mp3"
      rm "$jpg"
    else
      echo "❌ Erreur ffmpeg sur $mp3"
      rm -f "${mp3}.tmp.mp3"
      
    fi
  fi
done
