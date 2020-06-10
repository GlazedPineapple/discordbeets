FROM rust:slim AS builder
WORKDIR /app

COPY install_deps.sh .
RUN ./install_deps.sh

COPY . .
RUN cargo build --release

FROM debian:10-slim
LABEL Author="Elliot Kovacs"
WORKDIR /app

COPY --from=builder /app/target/release/discordbeets /app

# Deps
RUN apt update && apt install ffmpeg curl -y
RUN curl -L https://yt-dl.org/downloads/latest/youtube-dl -o /usr/local/bin/youtube-dl && chmod a+rx /usr/local/bin/youtube-dl

CMD ["/app/discordbeets"]
