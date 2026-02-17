# ==========================================
# ÉTAPE 1 : Le Bûcheron (Rust Builder)
# ==========================================
FROM rust:alpine AS rust-builder
WORKDIR /usr/src/app

# Installation des libs statiques pour la compilation musl
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static sqlite-dev sqlite-static

COPY Cargo.toml Cargo.lock ./

# Création du main vide pour le cache des deps
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl

COPY src ./src

# On force la recompilation du vrai code
RUN touch src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl --features container

# ==========================================
# ÉTAPE 2 : Le Chimiste (Python Dependencies)
# ==========================================
FROM python:3.11-alpine AS python-builder
WORKDIR /build

COPY requirement.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirement.txt

# ==========================================
# ÉTAPE 3 : Le Livreur (Image Finale)
# ==========================================
FROM python:3.11-alpine
WORKDIR /app

# --- MODIF 1 : Ajout de bash ---
RUN apk add --no-cache ca-certificates openssl bash ffmpeg

# Copie des dépendances Python et du binaire Rust
COPY --from=python-builder /install /usr/local
COPY --from=rust-builder /usr/src/app/target/x86_64-unknown-linux-musl/release/palemachine /app/palemachine

# --- MODIF 2 : On ne copie PLUS config.toml ici (on le génère plus bas) ---
COPY .version ./
COPY bambam_morigatsu_chuapo ./bambam_morigatsu_chuapo
COPY pages ./pages
COPY downloader ./downloader

# --- MODIF 3 : Script de démarrage (Entrypoint) ---
# Ce script va créer config.toml avec tes variables AVANT de lancer l'app
# Format attendu : path = "..." (string) et port = 9999 (int)

RUN echo '#!/bin/bash' > /app/entrypoint.sh && \
    echo '# 1. Définition des valeurs par défaut' >> /app/entrypoint.sh && \
    echo ': "${PALE_PATH:=/app/downloads}"' >> /app/entrypoint.sh && \
    echo ': "${PALE_PORT:=9999}"' >> /app/entrypoint.sh && \
    echo '' >> /app/entrypoint.sh && \
    echo '# 2. Création du dossier de téléchargement' >> /app/entrypoint.sh && \
    echo 'mkdir -p "$PALE_PATH"' >> /app/entrypoint.sh && \
    echo '' >> /app/entrypoint.sh && \
    echo '# 3. Génération du config.toml' >> /app/entrypoint.sh && \
    echo 'echo "Génération de la config..."' >> /app/entrypoint.sh && \
    echo 'echo "path = \"$PALE_PATH\"" > config.toml' >> /app/entrypoint.sh && \
    echo 'echo "port = $PALE_PORT" >> config.toml' >> /app/entrypoint.sh && \
    echo '' >> /app/entrypoint.sh && \
    echo '# 4. Lancement de palemachine' >> /app/entrypoint.sh && \
    echo 'exec ./palemachine' >> /app/entrypoint.sh

# On rend le script et le binaire exécutables
RUN chmod +x /app/entrypoint.sh /app/palemachine

# Documentation du port
EXPOSE 9999

# Le conteneur exécutera toujours ce script au démarrage
ENTRYPOINT ["/app/entrypoint.sh"]