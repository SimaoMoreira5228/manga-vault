# Manga Vault Docker Deployment

This docker-compose setup runs Manga Vault using the pre-built binaries from GitHub releases, providing a complete containerized deployment.

## Services

- **mysql**: MySQL 8.0 database
- **flaresolverr**: Anti-bot service for scraping
- **gql-api**: GraphQL API server
- **scheduler**: Background job scheduler for manga updates
- **website-server**: Web server serving the frontend

## Quick Start

1. **Clone the repository** (or download the docker-compose.yml and docker/ directory):
   ```bash
   git clone https://github.com/SimaoMoreira5228/manga-vault.git
   cd manga-vault
   ```

2. **Configure environment**:
   ```bash
   cp .env.example .env
   # Edit .env with your desired configuration
   ```

3. **Start the services**:
   ```bash
   docker-compose up -d
   ```

4. **Check logs**:
   ```bash
   docker-compose logs -f
   ```

5. **Access the application**:
   - Website: http://localhost:5227 (or your configured WEBSITE_PORT)
   - GraphQL API: http://localhost:5228 (or your configured GQL_API_PORT)

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and modify as needed:

- **Database**: Configure MySQL credentials and connection
- **Ports**: Set custom ports for services
- **Security**: Change JWT secret and CORS origins
- **Storage**: Configure upload folders and backup settings

### Volumes

The setup uses named volumes for persistent data:

- `mysql_data`: Database files
- `config_data`: Application configuration files
- `plugins_data`: Scraper plugins
- `uploads_data`: Uploaded manga files
- `website_data`: Static website files

## Production Deployment

For production:

1. **Use a reverse proxy** (nginx/caddy) in front of the website-server
2. **Configure SSL/TLS** certificates
3. **Set strong passwords** in .env
4. **Configure CORS** for your domain
5. **Set up backups** for the MySQL volume
6. **Monitor logs** and resource usage

## Updating

To update to new versions:

```bash
# Pull latest images and rebuild services
docker-compose pull
docker-compose up -d --build

# Or rebuild specific services
docker-compose up -d --build gql-api scheduler website-server
```

## Troubleshooting

### Check service health:

```bash
docker-compose ps
```

### View logs:

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f gql-api
```

### Database issues:

- Ensure MySQL has started properly before other services
- Check MySQL logs: `docker-compose logs mysql`

### Permission issues:

- The containers run as appropriate users
- Check volume permissions if needed

## Development

For development with local changes, you can mount the source code and build locally instead of using release binaries. Modify the Dockerfiles to copy source code and build instead of downloading releases.
