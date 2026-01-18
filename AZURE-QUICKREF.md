# Azure Deployment Quick Reference

Quick commands for deploying chess-rust to Microsoft Azure.

## Prerequisites

```bash
# Install Azure CLI
# macOS:
brew install azure-cli

# Linux:
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Verify installation
az --version
docker --version
```

## Quick Deploy with Docker Compose

```bash
# 1. Login to Azure
az login

# 2. Set variables
export RESOURCE_GROUP="chess-rust-rg"
export LOCATION="eastus"

# 3. Create resource group
az group create --name $RESOURCE_GROUP --location $LOCATION

# 4. Create and use Azure context
docker context create aci azure-chess-rust \
  --resource-group $RESOURCE_GROUP \
  --location $LOCATION

docker context use azure-chess-rust

# 5. Configure environment
cp .env.azure.example .env.azure
# Edit .env.azure as needed

# 6. Deploy
docker compose -f docker-compose.azure.yml --env-file .env.azure up

# 7. Get your URLs
az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-frontend-azure \
  --query ipAddress.fqdn \
  --output tsv
```

## Common Commands

### Deployment

```bash
# Deploy/update
docker compose -f docker-compose.azure.yml --env-file .env.azure up

# Deploy in background
docker compose -f docker-compose.azure.yml --env-file .env.azure up -d

# Stop deployment
docker compose -f docker-compose.azure.yml down
```

### Monitoring

```bash
# View backend logs
az container logs --resource-group $RESOURCE_GROUP --name chess-backend-azure

# View frontend logs  
az container logs --resource-group $RESOURCE_GROUP --name chess-frontend-azure

# Stream logs in real-time
az container attach --resource-group $RESOURCE_GROUP --name chess-backend-azure

# Check container status
az container list --resource-group $RESOURCE_GROUP --output table
```

### Management

```bash
# Restart a container
az container restart --resource-group $RESOURCE_GROUP --name chess-backend-azure

# Get container details
az container show --resource-group $RESOURCE_GROUP --name chess-backend-azure

# Stop a container
az container stop --resource-group $RESOURCE_GROUP --name chess-backend-azure

# Start a container
az container start --resource-group $RESOURCE_GROUP --name chess-backend-azure
```

### Testing

```bash
# Get backend URL
BACKEND_URL=$(az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure \
  --query ipAddress.fqdn -o tsv)

# Test health endpoint
curl http://$BACKEND_URL:3000/health

# Create a game
curl -X POST http://$BACKEND_URL:3000/games

# Get frontend URL
FRONTEND_URL=$(az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-frontend-azure \
  --query ipAddress.fqdn -o tsv)

echo "Frontend: http://$FRONTEND_URL"
```

## Cleanup

```bash
# Switch back to default context
docker context use default

# Remove containers
docker compose -f docker-compose.azure.yml down

# Remove Azure context
docker context rm azure-chess-rust

# Delete resource group (removes all resources)
az group delete --name $RESOURCE_GROUP --yes --no-wait
```

## Alternative: Azure CLI Method

### Deploy with Azure Container Registry

```bash
# Create container registry
ACR_NAME="chessrustacr"  # Must be globally unique
az acr create \
  --resource-group $RESOURCE_GROUP \
  --name $ACR_NAME \
  --sku Basic

# Build and push images
az acr build --registry $ACR_NAME --image chess-backend:latest .
az acr build --registry $ACR_NAME --image chess-frontend:latest ./frontend

# Get credentials
ACR_PASSWORD=$(az acr credential show --name $ACR_NAME --query passwords[0].value -o tsv)

# Deploy backend
az container create \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend \
  --image $ACR_NAME.azurecr.io/chess-backend:latest \
  --registry-password $ACR_PASSWORD \
  --dns-name-label chess-backend-$RANDOM \
  --ports 3000 \
  --cpu 1 --memory 1.5

# Get backend URL
BACKEND_FQDN=$(az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend \
  --query ipAddress.fqdn -o tsv)

# Deploy frontend
az container create \
  --resource-group $RESOURCE_GROUP \
  --name chess-frontend \
  --image $ACR_NAME.azurecr.io/chess-frontend:latest \
  --registry-password $ACR_PASSWORD \
  --dns-name-label chess-frontend-$RANDOM \
  --ports 80 \
  --cpu 0.5 --memory 0.5 \
  --environment-variables API_URL=http://$BACKEND_FQDN:3000
```

## Troubleshooting

```bash
# Check container state
az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure \
  --query instanceView.state

# View events
az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure \
  --query instanceView.events

# Test from inside Azure
az container exec \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure \
  --exec-command "/bin/sh"
```

## Validation

```bash
# Run validation script
./validate-azure.sh

# Test Docker Compose config
docker compose -f docker-compose.azure.yml config

# Test local build
docker build -t test-backend .
docker build -t test-frontend ./frontend
```

## Context Management

```bash
# List contexts
docker context ls

# Show current context
docker context show

# Switch contexts
docker context use azure-chess-rust  # Azure
docker context use default           # Local

# Remove context
docker context rm azure-chess-rust
```

## Environment Variables

Key variables in `.env.azure`:

```bash
BACKEND_PORT=3000
FRONTEND_PORT=80
RUST_LOG=info
API_URL=http://your-backend-dns:3000
```

## Cost Estimate

Approximate monthly costs (24/7 operation):

- **Backend** (1 CPU, 1.5 GB): ~$32/month
- **Frontend** (0.5 CPU, 0.5 GB): ~$16/month
- **Total**: ~$48/month

Save costs by stopping containers when not in use.

## Additional Resources

- [Full Azure Deployment Guide](AZURE-DEPLOYMENT.md)
- [Azure Container Instances Docs](https://docs.microsoft.com/en-us/azure/container-instances/)
- [Docker Azure Integration](https://docs.docker.com/cloud/aci-integration/)
- [Azure CLI Reference](https://docs.microsoft.com/en-us/cli/azure/)
