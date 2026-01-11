# Use nginx to serve the static files
FROM nginx:alpine

# Copy the frontend files to nginx html directory
COPY index.html /usr/share/nginx/html/
COPY styles.css /usr/share/nginx/html/
COPY app.js /usr/share/nginx/html/
# Note: config.js is copied as a fallback and is normally overwritten
# at container startup by docker-entrypoint.sh, which generates the
# runtime configuration used by the frontend.
COPY config.js /usr/share/nginx/html/
COPY README.md /usr/share/nginx/html/

# Copy custom nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy entrypoint script
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

# Expose port 80
EXPOSE 80

# Start nginx with entrypoint script
ENTRYPOINT ["/docker-entrypoint.sh"]
