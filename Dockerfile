FROM rust:1.62.1 as builder
WORKDIR /usr/src/threematrix
COPY . .
RUN cargo fetch --locked
RUN cargo build --release --frozen --offline

FROM rust:1.62.1
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/threematrix/target/release/threematrix /usr/bin/
WORKDIR /config
CMD ["threematrix"]
