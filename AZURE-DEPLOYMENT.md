# Azure Deployment Guide

This guide explains how to deploy the chess-rust application to Microsoft Azure using Docker Compose with Azure Container Instances (ACI).

## Overview

Azure provides multiple ways to deploy containerized applications. This guide focuses on deploying using Docker Compose with Azure Container Instances, which offers:

- **Easy deployment**: Use familiar Docker Compose syntax
- **Serverless**: No need to manage VMs or Kubernetes clusters
- **Cost-effective**: Pay only for what you use (per-second billing)
- **Quick scaling**: Deploy and scale containers quickly
- **Integrated networking**: Automatic DNS and networking setup

## Prerequisites

1. **Azure Account**: [Sign up for Azure](https://azure.microsoft.com/free/) (free tier includes $200 credit for 30 days)
2. **Azure CLI**: [Install Azure CLI](https://docs.microsoft.com/en-us/cli/azure/install-azure-cli)
3. **Docker CLI with Azure integration**: Install Docker Desktop or Docker CLI with Azure support
4. **Git**: To clone the repository

### Install Azure CLI

```bash
# macOS
brew install azure-cli

# Linux (Ubuntu/Debian)
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Windows
# Download from https://aka.ms/installazurecliwindows
```

### Verify Installation

```bash
az --version
docker --version
```

## Deployment Methods

### Method 1: Azure Container Instances with Docker Compose (Recommended)

This method uses Docker Compose syntax to deploy to Azure Container Instances, providing a simple path from local development to cloud deployment.

#### Step 1: Login to Azure

```bash
# Login to Azure
az login

# Set your subscription (if you have multiple)
az account set --subscription "Your Subscription Name"

# Verify your subscription
az account show
```

#### Step 2: Create Azure Resources

```bash
# Set variables
RESOURCE_GROUP="chess-rust-rg"
LOCATION="eastus"  # Choose your preferred region

# Create resource group
az group create \
  --name $RESOURCE_GROUP \
  --location $LOCATION
```

#### Step 3: Configure Environment

```bash
# Copy and edit the Azure environment file
cp .env.azure.example .env.azure

# Edit .env.azure with your specific configuration
# Update API_URL after deployment with your actual Azure URL
```

#### Step 4: Create Azure Context for Docker

```bash
# Create an Azure container context
docker context create aci azure-chess-rust \
  --resource-group $RESOURCE_GROUP \
  --location $LOCATION

# Switch to Azure context
docker context use azure-chess-rust

# Verify context
docker context ls
```

#### Step 5: Deploy with Docker Compose

```bash
# Deploy to Azure Container Instances
docker compose -f docker-compose.azure.yml --env-file .env.azure up

# This will:
# - Build your images (if not already built)
# - Push images to Azure Container Registry (automatically created)
# - Deploy containers to Azure Container Instances
# - Set up networking between containers
```

#### Step 6: Get Your Application URL

```bash
# Get the public IP or DNS name
az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-frontend-azure \
  --query ipAddress.fqdn \
  --output tsv

# Or get the IP address
az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-frontend-azure \
  --query ipAddress.ip \
  --output tsv
```

#### Step 7: Update Frontend Configuration

After getting your public URL, update the API_URL:

```bash
# Update .env.azure with your backend URL
# API_URL=http://<your-dns-name>:3000

# Redeploy to apply changes
docker compose -f docker-compose.azure.yml --env-file .env.azure down
docker compose -f docker-compose.azure.yml --env-file .env.azure up
```

### Method 2: Azure Container Instances with Azure CLI

For more control, you can deploy directly using Azure CLI:

#### Step 1: Build and Push Images

```bash
# Create Azure Container Registry
ACR_NAME="chessrustacr"  # Must be globally unique
az acr create \
  --resource-group $RESOURCE_GROUP \
  --name $ACR_NAME \
  --sku Basic \
  --admin-enabled true

# Login to ACR
az acr login --name $ACR_NAME

# Build and push backend
docker build -t $ACR_NAME.azurecr.io/chess-backend:latest .
docker push $ACR_NAME.azurecr.io/chess-backend:latest

# Build and push frontend
docker build -t $ACR_NAME.azurecr.io/chess-frontend:latest ./frontend
docker push $ACR_NAME.azurecr.io/chess-frontend:latest
```

#### Step 2: Get ACR Credentials

```bash
# Get ACR credentials
ACR_USERNAME=$(az acr credential show --name $ACR_NAME --query username -o tsv)
ACR_PASSWORD=$(az acr credential show --name $ACR_NAME --query passwords[0].value -o tsv)
```

#### Step 3: Deploy Backend Container

```bash
# Deploy backend
az container create \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend \
  --image $ACR_NAME.azurecr.io/chess-backend:latest \
  --registry-login-server $ACR_NAME.azurecr.io \
  --registry-username $ACR_USERNAME \
  --registry-password $ACR_PASSWORD \
  --dns-name-label chess-backend-$RANDOM \
  --ports 3000 \
  --cpu 1 \
  --memory 1.5 \
  --environment-variables RUST_LOG=info

# Get backend URL
BACKEND_URL=$(az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend \
  --query ipAddress.fqdn \
  --output tsv)

echo "Backend URL: http://$BACKEND_URL:3000"
```

#### Step 4: Deploy Frontend Container

```bash
# Deploy frontend
az container create \
  --resource-group $RESOURCE_GROUP \
  --name chess-frontend \
  --image $ACR_NAME.azurecr.io/chess-frontend:latest \
  --registry-login-server $ACR_NAME.azurecr.io \
  --registry-username $ACR_USERNAME \
  --registry-password $ACR_PASSWORD \
  --dns-name-label chess-frontend-$RANDOM \
  --ports 80 \
  --cpu 0.5 \
  --memory 0.5 \
  --environment-variables API_URL=http://$BACKEND_URL:3000

# Get frontend URL
FRONTEND_URL=$(az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-frontend \
  --query ipAddress.fqdn \
  --output tsv)

echo "Frontend URL: http://$FRONTEND_URL"
```

### Method 3: Azure App Service (Web Apps for Containers)

For production workloads with more features:

```bash
# Create App Service Plan
az appservice plan create \
  --name chess-rust-plan \
  --resource-group $RESOURCE_GROUP \
  --is-linux \
  --sku B1

# Create backend web app
az webapp create \
  --resource-group $RESOURCE_GROUP \
  --plan chess-rust-plan \
  --name chess-backend-app \
  --deployment-container-image-name $ACR_NAME.azurecr.io/chess-backend:latest

# Configure backend
az webapp config appsettings set \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-app \
  --settings RUST_LOG=info

# Create frontend web app
az webapp create \
  --resource-group $RESOURCE_GROUP \
  --plan chess-rust-plan \
  --name chess-frontend-app \
  --deployment-container-image-name $ACR_NAME.azurecr.io/chess-frontend:latest

# Configure frontend with backend URL
az webapp config appsettings set \
  --resource-group $RESOURCE_GROUP \
  --name chess-frontend-app \
  --settings API_URL=https://chess-backend-app.azurewebsites.net
```

## Configuration

### Environment Variables

Configure in `.env.azure`:

```bash
# Backend
BACKEND_PORT=3000
RUST_LOG=info

# Frontend  
FRONTEND_PORT=80
API_URL=http://your-backend-dns:3000
```

### Health Monitoring

For Azure Container Instances, you can monitor container health using:

```bash
# Check container state
az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure \
  --query instanceView.state

# Test backend health endpoint
curl http://your-backend-url:3000/health

# View container logs for errors
az container logs \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure
```

**Note**: Health checks in Azure Container Instances are configured differently than in Docker Compose. When deploying to ACI, health probes are automatically configured based on the exposed ports.

### Resource Sizing

Recommended Azure Container Instances sizes:

**Backend:**
- CPU: 1.0 cores
- Memory: 1.5 GB
- Sufficient for moderate load (~100 concurrent games)

**Frontend:**
- CPU: 0.5 cores
- Memory: 0.5 GB
- Serves static files efficiently

## Monitoring and Management

### View Logs

```bash
# Backend logs
az container logs \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure

# Frontend logs
az container logs \
  --resource-group $RESOURCE_GROUP \
  --name chess-frontend-azure

# Stream logs in real-time
az container attach \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure
```

### Check Container Status

```bash
# List all containers in resource group
az container list \
  --resource-group $RESOURCE_GROUP \
  --output table

# Get detailed container info
az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure
```

### Restart Containers

```bash
# Restart a container
az container restart \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure
```

## Testing Your Deployment

### Test Backend API

```bash
# Set your backend URL
BACKEND_URL="http://your-backend-dns:3000"

# Health check
curl $BACKEND_URL/health

# Create a game
curl -X POST $BACKEND_URL/games

# List games
curl $BACKEND_URL/games/list
```

### Test Frontend

Open your browser and navigate to your frontend URL:
```
http://your-frontend-dns
```

You should see the chess game interface.

## Updating Your Deployment

### Update with Docker Compose

```bash
# Switch to Azure context
docker context use azure-chess-rust

# Rebuild and redeploy
docker compose -f docker-compose.azure.yml --env-file .env.azure up --build

# Or update specific service
docker compose -f docker-compose.azure.yml --env-file .env.azure up --build backend
```

### Update with Azure CLI

```bash
# Rebuild and push new images
docker build -t $ACR_NAME.azurecr.io/chess-backend:latest .
docker push $ACR_NAME.azurecr.io/chess-backend:latest

# Restart container to pull new image
az container restart \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend
```

## Cleanup

### Remove Docker Compose Deployment

```bash
# Switch to Azure context
docker context use azure-chess-rust

# Stop and remove containers
docker compose -f docker-compose.azure.yml down

# Switch back to default context
docker context use default

# Remove Azure context
docker context rm azure-chess-rust
```

### Remove All Azure Resources

```bash
# Delete the entire resource group (removes all resources)
az group delete \
  --name $RESOURCE_GROUP \
  --yes \
  --no-wait
```

## Cost Estimation

Azure Container Instances pricing (as of 2024):

**Backend (1 vCPU, 1.5 GB RAM):**
- ~$0.0000125/second = ~$32.40/month (running 24/7)

**Frontend (0.5 vCPU, 0.5 GB RAM):**
- ~$0.0000063/second = ~$16.20/month (running 24/7)

**Total**: ~$48.60/month for 24/7 operation

**Cost Optimization Tips:**
1. Stop containers when not in use (development)
2. Use Azure Container Apps for auto-scaling
3. Consider Azure App Service for production (better pricing at scale)
4. Use Azure Free tier for testing ($200 credit for 30 days)

## Production Considerations

### 1. HTTPS/SSL

For production, add HTTPS using Azure Application Gateway or Azure Front Door:

```bash
# Create Application Gateway with SSL termination
az network application-gateway create \
  --name chess-gateway \
  --resource-group $RESOURCE_GROUP \
  --location $LOCATION \
  --sku Standard_v2 \
  --capacity 2 \
  --frontend-port 443 \
  --http-settings-port 80 \
  --http-settings-protocol Http
```

### 2. Custom Domain

```bash
# Add custom domain to Application Gateway
az network application-gateway http-listener create \
  --gateway-name chess-gateway \
  --resource-group $RESOURCE_GROUP \
  --name chess-listener \
  --frontend-port 443 \
  --host-name chess.yourdomain.com
```

### 3. Persistent Storage

For game persistence, add Azure Database:

```bash
# Create Azure Database for PostgreSQL
az postgres flexible-server create \
  --resource-group $RESOURCE_GROUP \
  --name chess-postgres \
  --location $LOCATION \
  --admin-user chessadmin \
  --admin-password 'YourSecurePassword123!' \
  --sku-name Standard_B1ms \
  --tier Burstable \
  --storage-size 32
```

### 4. Monitoring

Enable Azure Monitor:

```bash
# Create Log Analytics workspace
az monitor log-analytics workspace create \
  --resource-group $RESOURCE_GROUP \
  --workspace-name chess-logs

# Enable container insights
az container create \
  --name chess-backend \
  --log-analytics-workspace chess-logs
```

### 5. Security

- Use Azure Key Vault for secrets
- Enable Azure DDoS Protection
- Configure Network Security Groups
- Implement rate limiting
- Add authentication (Azure AD B2C)

## Troubleshooting

### Container Won't Start

```bash
# Check container state
az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure \
  --query instanceView.state

# View logs for errors
az container logs \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure
```

### Cannot Connect to Backend

1. Verify backend is running:
   ```bash
   az container show \
     --resource-group $RESOURCE_GROUP \
     --name chess-backend-azure \
     --query ipAddress
   ```

2. Test health endpoint:
   ```bash
   curl http://your-backend-ip:3000/health
   ```

3. Check network security rules
4. Verify CORS configuration

### Image Pull Errors

```bash
# Verify ACR credentials
az acr login --name $ACR_NAME

# Check image exists
az acr repository list --name $ACR_NAME

# Verify container has correct image name
az container show \
  --resource-group $RESOURCE_GROUP \
  --name chess-backend-azure \
  --query containers[0].image
```

### High Costs

1. Check container resource allocation:
   ```bash
   az container show \
     --resource-group $RESOURCE_GROUP \
     --name chess-backend-azure \
     --query containers[0].resources
   ```

2. Stop unused containers:
   ```bash
   az container stop \
     --resource-group $RESOURCE_GROUP \
     --name chess-backend-azure
   ```

3. Consider scaling down or using different pricing tier

## Comparison with Other Platforms

### Azure vs Render

**Azure Advantages:**
- More control over infrastructure
- Better for enterprise workloads
- More compliance certifications
- Can integrate with other Azure services

**Render Advantages:**
- Simpler deployment
- Better for small projects
- Free tier doesn't sleep
- Easier for beginners

### Azure vs Docker (Local)

**Azure Advantages:**
- Publicly accessible
- No local machine needed
- Better for production
- Built-in monitoring and scaling

**Local Docker Advantages:**
- Free for development
- Faster iteration
- No internet connection needed
- Full control

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Deploy to Azure

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Login to Azure
        uses: azure/login@v1
        with:
          creds: ${{ secrets.AZURE_CREDENTIALS }}
      
      - name: Build and push images
        run: |
          az acr build --registry ${{ secrets.ACR_NAME }} \
            --image chess-backend:latest \
            --file Dockerfile .
          
          az acr build --registry ${{ secrets.ACR_NAME }} \
            --image chess-frontend:latest \
            --file frontend/Dockerfile \
            ./frontend
      
      - name: Deploy to Azure Container Instances
        run: |
          az container restart \
            --resource-group ${{ secrets.RESOURCE_GROUP }} \
            --name chess-backend
          
          az container restart \
            --resource-group ${{ secrets.RESOURCE_GROUP }} \
            --name chess-frontend
```

## Additional Resources

- [Azure Container Instances Documentation](https://docs.microsoft.com/en-us/azure/container-instances/)
- [Docker Azure Integration](https://docs.docker.com/cloud/aci-integration/)
- [Azure CLI Reference](https://docs.microsoft.com/en-us/cli/azure/)
- [Chess-Rust Project Documentation](README.md)
- [Docker Deployment Guide](DOCKER-DEPLOYMENT.md)
- [Render Deployment Guide](RENDER-DEPLOYMENT.md)

## Support

For issues or questions:
1. Check Azure Container Instances documentation
2. Run the validation script: `./validate-azure.sh`
3. Check container logs: `az container logs`
4. Review GitHub issues

## License

See LICENSE file for details.
