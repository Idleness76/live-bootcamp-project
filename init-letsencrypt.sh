#!/bin/bash

domains=(idlelgr.duckdns.org)
rsa_key_size=4096
data_path="./certbot"
email="${LETSENCRYPT_EMAIL:-admin@idlelgr.duckdns.org}"  # Use env var or fallback
staging=0 # Set to 1 for testing to avoid rate limits

echo "### [1/7] Creating certbot directories..."
mkdir -p "$data_path/conf"
mkdir -p "$data_path/www"
echo "✓ Certbot directories created successfully"

echo "### [2/7] Downloading recommended TLS parameters..."
curl -s https://raw.githubusercontent.com/certbot/certbot/master/certbot-nginx/certbot_nginx/_internal/tls_configs/options-ssl-nginx.conf > "$data_path/conf/options-ssl-nginx.conf"
curl -s https://raw.githubusercontent.com/certbot/certbot/master/certbot/certbot/ssl-dhparams.pem > "$data_path/conf/ssl-dhparams.pem"
echo "✓ TLS parameters downloaded successfully"

echo "### [3/7] Creating dummy certificate for idlelgr.duckdns.org..."
path="/etc/letsencrypt/live/idlelgr.duckdns.org"
mkdir -p "$data_path/conf/live/idlelgr.duckdns.org"
docker compose run --rm --entrypoint "\
  openssl req -x509 -nodes -newkey rsa:$rsa_key_size -days 1\
    -keyout '$path/privkey.pem' \
    -out '$path/fullchain.pem' \
    -subj '/CN=localhost'" certbot
echo "✓ Dummy certificate created successfully"

echo "### [4/7] Starting nginx with dummy certificate..."
docker compose up --force-recreate -d nginx
echo "✓ Nginx started successfully"

echo "### [5/7] Cleaning up dummy certificate..."
docker compose run --rm --entrypoint "\
  rm -Rf /etc/letsencrypt/live/idlelgr.duckdns.org && \
  rm -Rf /etc/letsencrypt/archive/idlelgr.duckdns.org && \
  rm -Rf /etc/letsencrypt/renewal/idlelgr.duckdns.org.conf" certbot
echo "✓ Dummy certificate removed successfully"

echo "### [6/7] Requesting production Let's Encrypt certificate..."
docker compose run --rm --entrypoint "\
  certbot certonly --webroot -w /var/www/certbot \
    --email $email \
    -d idlelgr.duckdns.org \
    --rsa-key-size $rsa_key_size \
    --agree-tos \
    --force-renewal" certbot
echo "✓ SSL certificate obtained successfully"

echo "### [7/7] Reloading nginx with new certificate..."
docker compose exec nginx nginx -s reload
echo "✓ Nginx reloaded successfully"
echo ""
echo "🎉 SSL certificate setup completed! Your site is now secured with HTTPS."