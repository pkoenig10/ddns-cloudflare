FROM rust:1.50.0 AS builder

COPY . /app

WORKDIR /app
RUN cargo build --release

FROM gcr.io/distroless/cc

COPY --from=builder /app/target/release/ddns /

ENTRYPOINT ["/ddns"]
