#!/bin/sh
set -e

# Generate config.js with environment variables
# The API_URL environment variable can be set when starting the container
api_url_default='http://localhost:3000'
api_url_raw="${API_URL:-$api_url_default}"

# Basic validation: only allow http/https URLs, otherwise fall back to default
case "$api_url_raw" in
  http://*|https://*)
    api_url_valid="$api_url_raw"
    ;;
  *)
    api_url_valid="$api_url_default"
    ;;
esac

# Escape characters that are unsafe in a single-quoted JavaScript string
# - escape backslashes
# - escape single quotes
safe_api_url_escaped=$(printf '%s' "$api_url_valid" | sed "s/\\\\/\\\\\\\\/g; s/'/\\\\'/g")

cat > /usr/share/nginx/html/config.js << EOF
// Configuration loaded from environment at container startup
window.chessConfig = {
    apiUrl: '${safe_api_url_escaped}'
};
EOF

# Start nginx
exec nginx -g "daemon off;"
