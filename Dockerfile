FROM --platform=$BUILDPLATFORM rust:1.88.0 AS builder

RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    gcc-x86-64-linux-gnu
RUN rustup target add \
    aarch64-unknown-linux-gnu \
    x86_64-unknown-linux-gnu

COPY . /app
WORKDIR /app

ARG TARGETPLATFORM

RUN case $TARGETPLATFORM in \
        linux/amd64) TARGET=x86_64-unknown-linux-gnu ;; \
        linux/arm64) TARGET=aarch64-unknown-linux-gnu ;; \
    esac && \

    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    cargo build --release --target $TARGET && \

    cp target/$TARGET/release/ddns-cloudflare .

FROM gcr.io/distroless/cc-debian12:latest@sha256:eccec5274132c1be0ce5d2c8e6fe41033e64af5e987ccee9007826e4c012069d

COPY --from=builder /app/ddns-cloudflare /

ENTRYPOINT ["/ddns-cloudflare"]
