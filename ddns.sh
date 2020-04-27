#!/usr/bin/env bash

set -o pipefail

CF_API=https://api.cloudflare.com/client/v4

cloudflare() {
  curl -sSL \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  "$@"
}

get_dns_server() {
  case $1 in
    4)
      echo 1.1.1.1
      ;;
    6)
      echo 2606:4700:4700::1111
      ;;
  esac
}

get_ip_address() {
  dig +short @$(get_dns_server $1) ch txt whoami.cloudflare | tr -d '"'
}

get_zone_id() {
  cloudflare "$CF_API/zones?name=$DOMAIN" | jq -er '.result[0].id'
}

get_dns_record_id() {
  cloudflare "$CF_API/zones/$ZONE_ID/dns_records?type=$1&name=$DOMAIN" | jq -er '.result[0].id'
}

get_dns_record_content() {
  cloudflare "$CF_API/zones/$ZONE_ID/dns_records/$1" | jq -er '.result.content'
}

update_dns_record() {
  cloudflare -X PATCH "$CF_API/zones/$ZONE_ID/dns_records/$1" -d "{\"content\":\"$2\"}" | jq -er '.result.id' > /dev/null
}

ddns() {
  ip_address=$(get_ip_address $1)
  if [ $? -ne 0 ]; then
    echo "Failed to get IPv$1 address"
    return 1
  fi

  dns_record_id=$(get_dns_record_id $2)
  if [ $? -ne 0 ]; then
    echo "$DOMAIN $2 record not found"
    return 0
  fi

  dns_record_content=$(get_dns_record_content $dns_record_id)
  if [ $? -ne 0 ]; then
    echo "Failed to get $DOMAIN $2 record content"
    return 1
  fi

  if [ "$ip_address" == "$dns_record_content" ]; then
    echo "No update required for $DOMAIN $2 record ($dns_record_content)"
    return 0
  fi

  echo "Updating $DOMAIN $2 record from $dns_record_content to $ip_address..."

  update_dns_record $dns_record_id $ip_address
  if [ $? -ne 0 ]; then
    echo "Failed to update $DOMAIN $2 record"
    return 1
  fi

  echo "Updated $DOMAIN $2 record"
}

ZONE_ID=$(get_zone_id)
if [ $? -ne 0 ]; then
  echo "Zone for $DOMAIN was not found"
  exit 0
fi

ddns 4 A
ddns 6 AAAA
