# ddns-cloudflare

[![](https://github.com/pkoenig10/ddns-cloudflare/actions/workflows/ci.yml/badge.svg)][actions]

A DDNS service using [Cloudflare DNS](https://www.cloudflare.com/dns/).

[actions]: https://github.com/pkoenig10/ddns-cloudflare/actions

## Configuration

### Environment variables

| Variable | Description | Required? | Default |
|:-|:-|:-:|:-:|
| `CLOUDFLARE_API_TOKEN` | The Cloudflare API token. Requires `Zone:Read`, `DNS:Read`, and `DNS:Edit`. | Yes | - |
| `DOMAIN` | The Cloudflare zone domain name. | Yes | - |
