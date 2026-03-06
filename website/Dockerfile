# Production image with nginx for serving pre-built static assets
# Build locally with: trunk build --release
FROM nginx:1.27-alpine

# Remove default nginx config and html
RUN rm -rf /etc/nginx/conf.d/default.conf /usr/share/nginx/html/*

# Copy custom nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Copy pre-built dist files (run `trunk build --release` locally first)
COPY dist/ /usr/share/nginx/html

# Remove Trunk hot-reload script from production HTML (if present)
# The script starts with <script>"use strict"; and ends before </body>
RUN if grep -q '__TRUNK_ADDRESS__' /usr/share/nginx/html/index.html; then \
        awk '/<script>"use strict";/{found=1} !found; /<\/script><\/body>/{found=0; print "</body>"}' \
            /usr/share/nginx/html/index.html > /tmp/index.html && \
        mv /tmp/index.html /usr/share/nginx/html/index.html; \
    fi

# Pre-compress static assets with gzip for nginx gzip_static
RUN find /usr/share/nginx/html -type f \( -name "*.html" -o -name "*.css" -o -name "*.js" -o -name "*.wasm" -o -name "*.svg" -o -name "*.json" -o -name "*.xml" -o -name "*.txt" \) \
    -exec gzip -9 -k {} \;

# Create non-root user for security
RUN adduser -D -u 1000 appuser \
    && chown -R appuser:appuser /usr/share/nginx/html \
    && chown -R appuser:appuser /var/cache/nginx \
    && chown -R appuser:appuser /var/log/nginx \
    && touch /var/run/nginx.pid \
    && chown appuser:appuser /var/run/nginx.pid

USER appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
