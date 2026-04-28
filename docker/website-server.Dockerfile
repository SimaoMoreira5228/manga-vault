FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY manga-vault-website-server-linux-x86_64 ./manga-vault-website-server
COPY website/ ./website/

RUN chmod +x manga-vault-website-server && mkdir -p config

EXPOSE 5227

CMD ["./manga-vault-website-server"]
