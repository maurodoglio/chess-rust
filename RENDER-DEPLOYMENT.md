# Render Deployment Guide

This guide explains how to deploy the chess-rust application (both backend and frontend) to [Render](https://render.com/).

## Overview

The application consists of two services:
1. **Backend**: Rust web service (Docker-based)
2. **Frontend**: Static site (HTML/CSS/JS)

Both services are configured via the `render.yaml` file in the repository root for easy deployment.

## Prerequisites

1. A [Render account](https://dashboard.render.com/register) (free tier available)
2. Your chess-rust repository on GitHub

## Deployment Methods

### Method 1: Using render.yaml (Recommended)

This is the easiest method as it uses Infrastructure as Code to deploy both services automatically.

1. **Fork or clone this repository** to your GitHub account

2. **Connect your repository to Render**:
   - Log in to [Render Dashboard](https://dashboard.render.com/)
   - Click "New +" and select "Blueprint"
   - Connect your GitHub account if you haven't already
   - Select the chess-rust repository
   - Render will automatically detect the `render.yaml` file

3. **Review and Deploy**:
   - Render will show you the services that will be created:
     - `chess-rust-backend` - Web Service (Docker)
     - `chess-rust-frontend` - Static Site
   - Click "Apply" to create both services
   - The backend will build using Docker (this takes 5-10 minutes on first deploy)
   - The frontend will be deployed as a static site

4. **Access your application**:
   - Backend API: `https://chess-rust-backend.onrender.com`
   - Frontend UI: `https://chess-rust-frontend.onrender.com`
   - The frontend is automatically configured to connect to the backend

### Method 2: Manual Deployment

If you prefer to deploy services manually:

#### Deploy Backend

1. Go to [Render Dashboard](https://dashboard.render.com/)
2. Click "New +" and select "Web Service"
3. Connect your repository
4. Configure the service:
   - **Name**: `chess-rust-backend`
   - **Runtime**: Docker
   - **Region**: Oregon (or your preferred region)
   - **Plan**: Free
   - **Docker Build Context**: `.` (root directory)
   - **Dockerfile Path**: `./Dockerfile`
5. Add environment variables:
   - `PORT`: `10000` (automatically set by Render)
   - `RUST_LOG`: `info`
6. Set health check path: `/health`
7. Click "Create Web Service"

#### Deploy Frontend

1. Go to [Render Dashboard](https://dashboard.render.com/)
2. Click "New +" and select "Static Site"
3. Connect your repository
4. Configure the service:
   - **Name**: `chess-rust-frontend`
   - **Build Command**: `cd frontend && ./build.sh`
   - **Publish Directory**: `./frontend`
5. Add environment variable:
   - `API_URL`: `https://chess-rust-backend.onrender.com` (use your actual backend URL)
6. Click "Create Static Site"

## Configuration Details

### Backend Configuration

The backend service:
- Runs on the port specified by the `PORT` environment variable (Render sets this automatically)
- Uses the existing `Dockerfile` for containerized deployment
- Includes a `/health` endpoint for health checks
- Logs are available in the Render dashboard

### Frontend Configuration

The frontend:
- Is served as static HTML/CSS/JS files
- Uses a build script (`build.sh`) to inject the backend API URL at build time
- Automatically connects to the backend service via the `API_URL` environment variable
- Includes URL rewriting so that all routes serve `index.html` (SPA-friendly)

### Environment Variables

#### Backend
- `PORT` - Server port (automatically set by Render, defaults to 10000)
- `RUST_LOG` - Logging level (e.g., `info`, `debug`)

#### Frontend
- `API_URL` - Backend API URL (automatically set from backend service when using render.yaml)

## Free Tier Limitations

Render's free tier includes:
- ✅ 750 hours/month of runtime per service
- ✅ Automatic HTTPS
- ✅ Continuous deployment from Git
- ⚠️ Services spin down after 15 minutes of inactivity (cold starts on next request)
- ⚠️ 512 MB RAM limit
- ⚠️ Shared CPU

**Note**: The backend will experience a ~30 second cold start delay when inactive for 15+ minutes. This is normal for free tier services.

## Updating Your Deployment

Both services support continuous deployment:

1. **Automatic**: Push changes to your repository's main branch
   - Render automatically detects changes and redeploys
   - No manual intervention needed

2. **Manual**: Use the Render dashboard
   - Go to your service
   - Click "Manual Deploy" → "Deploy latest commit"

## Monitoring and Logs

### View Logs
1. Go to [Render Dashboard](https://dashboard.render.com/)
2. Select your service (backend or frontend)
3. Click on the "Logs" tab
4. View real-time logs and historical entries

### Health Checks
The backend includes a health check endpoint:
```bash
curl https://chess-rust-backend.onrender.com/health
# Returns: OK
```

Render automatically monitors this endpoint and will restart the service if it becomes unhealthy.

## Testing Your Deployment

### Test Backend API
```bash
# Health check
curl https://chess-rust-backend.onrender.com/health

# Create a game
curl -X POST https://chess-rust-backend.onrender.com/games

# List games
curl https://chess-rust-backend.onrender.com/games/list
```

### Test Frontend
Open your browser and navigate to:
```
https://chess-rust-frontend.onrender.com
```

You should see the chess game interface. Try:
1. Creating a new game
2. Opening the same game in another browser/tab
3. Making moves from both sides

## Custom Domains

To use a custom domain:

1. Go to your service settings in Render dashboard
2. Click "Custom Domain"
3. Add your domain (e.g., `chess.yourdomain.com`)
4. Configure DNS as instructed by Render
5. Render automatically provisions SSL certificates

## Troubleshooting

### Backend won't start
- Check the build logs in Render dashboard
- Verify the Dockerfile builds locally: `docker build -t test .`
- Check environment variables are set correctly
- Look for error messages in the service logs

### Frontend can't connect to backend
- Verify the `API_URL` environment variable is set correctly
- Check that the backend service is running and healthy
- Open browser developer console to see network errors
- Verify CORS is enabled (it is by default in the code)

### Cold Start Delays
- This is expected on the free tier after 15 minutes of inactivity
- Consider upgrading to a paid plan for always-on services
- First request after inactivity will take ~30 seconds

### Build Failures
- Check that all dependencies are specified in `Cargo.toml`
- Verify the Dockerfile builds successfully locally
- Check Render's build logs for specific error messages

## Security Considerations

1. **CORS**: The backend allows all origins by default. For production, restrict this to your frontend domain.

2. **Rate Limiting**: Consider adding rate limiting to prevent abuse.

3. **Authentication**: The current implementation has no authentication. Add this for production use.

4. **Environment Variables**: Never commit secrets to the repository. Use Render's environment variable feature.

## Cost Optimization

### Free Tier Tips
- Both services can run on the free tier
- Backend spins down after 15 minutes of inactivity (cold starts)
- Frontend (static site) doesn't spin down

### Paid Plans
For production use, consider:
- **Starter Plan** ($7/month per service): No spin down, better performance
- **Standard Plan** ($25/month per service): More resources, faster builds

## Support

- [Render Documentation](https://render.com/docs)
- [Render Community Forum](https://community.render.com/)
- [Chess-Rust GitHub Issues](https://github.com/maurodoglio/chess-rust/issues)

## Next Steps

After deployment:
1. Test all functionality (create game, join game, make moves)
2. Monitor logs for errors
3. Set up custom domain if desired
4. Consider adding monitoring/alerting
5. Review security settings for production use

## Alternative: Deploy Backend Only

If you want to deploy only the backend and run the frontend locally:

1. Deploy just the backend using Method 2 above
2. Update `frontend/config.js` locally:
   ```javascript
   window.chessConfig = {
       apiUrl: 'https://your-backend.onrender.com'
   };
   ```
3. Serve the frontend locally:
   ```bash
   cd frontend
   python3 -m http.server 8000
   ```

## Migration from Docker/Other Platforms

If you're migrating from Docker Compose or another platform:

1. The application architecture remains the same
2. Backend uses the same Docker image
3. Frontend serves the same static files
4. Only environment variable configuration differs
5. No code changes required!

The main difference is that Render manages the container orchestration, networking, and HTTPS for you.
