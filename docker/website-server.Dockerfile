FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    jq \
    unzip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN LATEST_RELEASE=$(curl -s https://api.github.com/repos/SimaoMoreira5228/manga-vault/releases/latest | jq -r '.tag_name') && \
    curl -L -o manga-vault-website-server \
    "https://github.com/SimaoMoreira5228/manga-vault/releases/download/${LATEST_RELEASE}/manga-vault-website-server-linux-x86_64" && \
    chmod +x manga-vault-website-server

RUN mkdir -p website && \
    LATEST_RELEASE=$(curl -s https://api.github.com/repos/SimaoMoreira5228/manga-vault/releases/latest | jq -r '.tag_name') && \
    curl -L -o website.zip \
    "https://github.com/SimaoMoreira5228/manga-vault/releases/download/${LATEST_RELEASE}/website.zip" && \
    unzip website.zip -d website/ && \
    rm website.zip

RUN mkdir -p config

EXPOSE 5227

CMD ["./manga-vault-website-server"]
