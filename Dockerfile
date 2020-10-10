FROM alpine:3.12

RUN apk add --no-cache \
    bind-tools \
    curl \
    jq

COPY ddns.sh /

ENTRYPOINT ["/ddns.sh"]
