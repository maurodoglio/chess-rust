# Quick Start: Deploy to Render

Get your chess game running in the cloud in under 5 minutes!

## 🚀 One-Click Deployment

1. **Fork this repository** to your GitHub account

2. **Update render.yaml** (Important for forks!):
   - Open `render.yaml` in your fork
   - Change the `repo:` URL on lines 7 and 22 to your fork:
     ```yaml
     repo: https://github.com/YOUR-USERNAME/chess-rust
     ```
   - Commit the change

3. **Sign up for Render** (free): https://dashboard.render.com/register

4. **Deploy with Blueprint**:
   - Click "New +" in Render dashboard
   - Select "Blueprint"
   - Connect your GitHub account
   - Choose your forked `chess-rust` repository
   - Click "Apply"

5. **Done!** 🎉
   - Backend API: `https://chess-rust-backend.onrender.com`
   - Frontend: `https://chess-rust-frontend.onrender.com`

## ⏱️ What to Expect

- **First Deploy**: ~5-10 minutes (builds Docker image)
- **Subsequent Deploys**: ~2-3 minutes
- **Free Tier**: Services spin down after 15 minutes (30s cold start)

## 🔧 Customization

### Change Service Names
Edit `render.yaml`:
```yaml
name: my-custom-backend  # Change this
```

### Update for Your Fork
Edit `render.yaml`:
```yaml
repo: https://github.com/YOUR-USERNAME/chess-rust
```

### Environment Variables
Set in Render Dashboard → Service Settings → Environment:
- Backend: `RUST_LOG=debug` for verbose logging
- Frontend: `API_URL` (auto-set from backend)

## 📚 Need More Help?

See [RENDER-DEPLOYMENT.md](RENDER-DEPLOYMENT.md) for:
- Manual deployment steps
- Troubleshooting guide
- Security best practices
- Custom domain setup
- Monitoring and logs

## ✅ Validate Before Deploy

```bash
./validate-render.sh
```

## 🆓 Free Tier Limits

- ✅ Automatic HTTPS
- ✅ Continuous deployment
- ✅ 750 hours/month per service
- ⚠️ Services spin down after 15 min inactivity
- ⚠️ 512 MB RAM per service

Need always-on services? Upgrade to Starter plan ($7/month).

## 🧪 Test Your Deployment

```bash
# Check backend health
curl https://chess-rust-backend.onrender.com/health

# Create a game
curl -X POST https://chess-rust-backend.onrender.com/games
```

Then open the frontend URL in your browser to play!

---

**Questions?** See the full [RENDER-DEPLOYMENT.md](RENDER-DEPLOYMENT.md) guide.
