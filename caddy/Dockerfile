FROM caddy:builder-alpine AS builder

RUN xcaddy build \
    --with github.com/mholt/caddy-l4/modules/l4ssh \
    --with github.com/WeidiDeng/caddy-cloudflare-ip \
    --with github.com/enum-gg/caddy-discord \
    --with github.com/mholt/caddy-grpc-web


FROM caddy:alpine

COPY --from=builder /usr/bin/caddy /usr/bin/caddy