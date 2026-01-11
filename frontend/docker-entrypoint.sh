#!/bin/sh
set -e

# Generate config.js with environment variables
cat > /usr/share/nginx/html/config.js << EOF
// Configuration loaded from environment
window.ENV_API_URL = '${API_URL:-http://localhost:3000}';
window.chessConfig = {
    apiUrl: window.ENV_API_URL
};
EOF

# Start nginx
exec nginx -g "daemon off;"
