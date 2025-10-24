FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    jq \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN LATEST_RELEASE=$(curl -s https://api.github.com/repos/SimaoMoreira5228/manga-vault/releases/latest | jq -r '.tag_name') && \
    curl -L -o manga-vault-gql \
    "https://github.com/SimaoMoreira5228/manga-vault/releases/download/${LATEST_RELEASE}/manga-vault-gql-linux-x86_64" && \
    chmod +x manga-vault-gql

RUN mkdir -p config plugins uploads

EXPOSE 5228

CMD ["./manga-vault-gql"]
