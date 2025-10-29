FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    jq \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN RELEASES=$(curl -s "https://api.github.com/repos/SimaoMoreira5228/manga-vault/releases?per_page=100") && \
    LATEST_SCHEDULER=$(echo "$RELEASES" | jq -r '[.[] | select(.tag_name | startswith("scheduler@"))] | max_by(.published_at) | .tag_name') && \
    curl -L -o manga-vault-scheduler \
    "https://github.com/SimaoMoreira5228/manga-vault/releases/download/${LATEST_SCHEDULER}/manga-vault-scheduler-linux-x86_64" && \
    chmod +x manga-vault-scheduler

RUN mkdir -p config plugins uploads

CMD ["./manga-vault-scheduler"]