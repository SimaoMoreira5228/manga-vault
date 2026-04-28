FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY manga-vault-gql-linux-x86_64 ./manga-vault-gql

RUN chmod +x manga-vault-gql && mkdir -p config plugins uploads

EXPOSE 5228

CMD ["./manga-vault-gql"]
