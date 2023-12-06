# local-gradient/Dockerfile
FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build --release

CMD ["target/release/local-gradient"]