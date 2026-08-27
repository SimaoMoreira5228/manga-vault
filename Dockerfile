# syntax=docker/dockerfile:1.7
FROM rust:1-slim AS build
WORKDIR /src
RUN apt-get update \
	&& apt-get install -y --no-install-recommends pkg-config libssl-dev mold \
	&& rm -rf /var/lib/apt/lists/*
COPY .cargo .cargo
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY apps apps
COPY tools tools
RUN --mount=type=cache,id=manga-vault-cargo-registry,target=/usr/local/cargo/registry \
	--mount=type=cache,id=manga-vault-cargo-git,target=/usr/local/cargo/git \
	--mount=type=cache,id=manga-vault-target,target=/src/target \
	sh -c 'cargo build --release -p manga-vault-server && cp target/release/manga-vault /src/manga-vault'

FROM debian:trixie-slim
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates libssl3t64 \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=build /src/manga-vault /usr/local/bin/manga-vault
ENV DATABASE_URL="sqlite:///data/manga-vault.db?mode=rwc" \
	BIND_ADDR=0.0.0.0:8080 \
	PLUGINS_DIR=/plugins
VOLUME ["/data", "/plugins"]
EXPOSE 8080
ENTRYPOINT ["manga-vault"]
