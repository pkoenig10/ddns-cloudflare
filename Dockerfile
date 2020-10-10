FROM alpine:3.11

RUN apk add --no-cache \
    bind-tools \
    curl \
    jq

COPY ddns.sh /

ENTRYPOINT ["/ddns.sh"]
