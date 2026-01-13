#!/bin/bash
# Validation script to check Render deployment configuration

echo "🔍 Validating Render deployment configuration..."
echo ""

# Check if render.yaml exists
if [ ! -f "render.yaml" ]; then
    echo "❌ render.yaml not found"
    exit 1
else
    echo "✅ render.yaml exists"
fi

# Check if Dockerfile exists
if [ ! -f "Dockerfile" ]; then
    echo "❌ Dockerfile not found"
    exit 1
else
    echo "✅ Dockerfile exists"
fi

# Check if frontend build script exists and is executable
if [ ! -f "frontend/build.sh" ]; then
    echo "❌ frontend/build.sh not found"
    exit 1
elif [ ! -x "frontend/build.sh" ]; then
    echo "⚠️  frontend/build.sh exists but is not executable"
    echo "   Run: chmod +x frontend/build.sh"
else
    echo "✅ frontend/build.sh exists and is executable"
fi

# Check if frontend files exist
if [ ! -f "frontend/index.html" ]; then
    echo "❌ frontend/index.html not found"
    exit 1
else
    echo "✅ frontend/index.html exists"
fi

# Check if main.rs has PORT support
if grep -q "std::env::var(\"PORT\")" src/main.rs; then
    echo "✅ Backend supports PORT environment variable"
else
    echo "❌ Backend does not support PORT environment variable"
    exit 1
fi

# Check if health endpoint exists in API
if grep -q "/health" src/api.rs; then
    echo "✅ Health check endpoint exists"
else
    echo "⚠️  Health check endpoint not found in api.rs"
fi

echo ""
echo "🎉 All validation checks passed!"
echo ""
echo "Next steps:"
echo "1. Push this repository to GitHub"
echo "2. Go to https://dashboard.render.com/"
echo "3. Create a new Blueprint and connect your repository"
echo "4. Render will automatically deploy both services"
echo ""
echo "For detailed instructions, see RENDER-DEPLOYMENT.md"
