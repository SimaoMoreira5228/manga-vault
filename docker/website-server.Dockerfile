FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    jq \
    unzip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN RELEASES=$(curl -s "https://api.github.com/repos/SimaoMoreira5228/manga-vault/releases?per_page=100") && \
    LATEST_SERVER=$(echo "$RELEASES" | jq -r '[.[] | select(.tag_name | startswith("website-server@"))] | max_by(.published_at) | .tag_name') && \
    curl -L -o manga-vault-website-server \
    "https://github.com/SimaoMoreira5228/manga-vault/releases/download/${LATEST_SERVER}/manga-vault-website-server-linux-x86_64" && \
    chmod +x manga-vault-website-server && \
    LATEST_WEBSITE=$(echo "$RELEASES" | jq -r '[.[] | select(.tag_name | startswith("website@"))] | max_by(.published_at) | .tag_name') && \
    mkdir -p website && \
    curl -L -o website.zip \
    "https://github.com/SimaoMoreira5228/manga-vault/releases/download/${LATEST_WEBSITE}/website.zip" && \
    unzip website.zip -d website/ && \
    rm website.zip

RUN mkdir -p config

EXPOSE 5227

CMD ["./manga-vault-website-server"]
