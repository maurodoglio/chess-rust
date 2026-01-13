#!/bin/bash
set -e

echo "Starting frontend build for Render..."

# The API_URL will be set by Render from the backend service
# If not set, use a placeholder
API_URL="${API_URL:-https://chess-rust-backend.onrender.com}"

echo "Using API_URL: $API_URL"

# Generate config.js with the API_URL from environment
cat > config.js << EOF
// Configuration loaded from environment at build time
window.chessConfig = {
    apiUrl: '${API_URL}'
};
EOF

echo "Frontend build complete!"
echo "Generated config.js with API URL: $API_URL"
