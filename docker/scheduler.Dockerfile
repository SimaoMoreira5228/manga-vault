FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY manga-vault-scheduler-linux-x86_64 ./manga-vault-scheduler

RUN chmod +x manga-vault-scheduler && mkdir -p config plugins uploads

CMD ["./manga-vault-scheduler"]
