#!/bin/sh
set -e

# Generate config.js with environment variables
# The API_URL environment variable can be set when starting the container
cat > /usr/share/nginx/html/config.js << EOF
// Configuration loaded from environment at container startup
window.chessConfig = {
    apiUrl: '${API_URL:-http://localhost:3000}'
};
EOF

# Start nginx
exec nginx -g "daemon off;"
