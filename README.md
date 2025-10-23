# URL Shortener

A high-performance URL shortener service built with modern Rust technologies. This service provides a simple API for creating shortened URLs and redirecting users to their original destinations.

## 🚀 Features

- **Fast URL shortening**: Generate short, unique identifiers for long URLs using nanoid
- **Reliable redirects**: Permanent redirects to original URLs with proper HTTP status codes
- **Rate limiting**: Built-in rate limiting to prevent abuse using tower-governor
- **Multi-database support**: SQLite and PostgreSQL database backends
- **Database abstraction**: Trait-based database layer for easy database switching
- **URL validation**: Input validation with configurable URL length limits (2048 characters)
- **Comprehensive logging**: Structured logging with tracing and request IDs
- **Health monitoring**: Built-in health check endpoint
- **API documentation**: OpenAPI 3.0 specification with interactive Swagger UI
- **Web interface**: Admin panel with Tera templates
- **API key protection**: Secure API endpoints with UUID-based authentication
- **Nix development environment**: Flake-based dev environment with pre-commit hooks
- **Production ready**: Built for deployment with graceful shutdown handling

## 🛠 Technology Stack

- **Framework**: [Axum](https://github.com/tokio-rs/axum) - Modern async web framework
- **Databases**: SQLite and PostgreSQL with [SQLx](https://github.com/launchbadge/sqlx) for type-safe queries
- **Rate Limiting**: [tower-governor](https://crates.io/crates/tower_governor) - Per-IP rate limiting with GCRA algorithm
- **Templates**: [Tera](https://keats.github.io/tera/) - Template engine for web interface
- **Configuration**: [Figment](https://github.com/SergioBenitez/figment) - Layered configuration
- **Logging**: Structured logging with `tracing` and Bunyan formatting
- **Development**: Nix flake with Fenix Rust toolchain and pre-commit hooks
- **Testing**: Comprehensive integration tests with in-memory databases

## 📡 API Endpoints

### Shorten a URL

POST /api/shorten
Content-Type: text/plain
x-api-key: YOUR_API_KEY

Example
curl -d 'https://www.google.com/'
-H "x-api-key: e4125dd1-3d3e-43a1-bc9c-dc0ba12ad4b5"
http://localhost:8000/api/shorten


**Response**: Returns a JSON response with shortened URL information

{
"success": true,
"message": "ok",
"status": 200,
"time": "2025-10-05T12:00:00Z",
"data": {
"shortened_url": "http://localhost:8000/AbC123",
"original_url": "https://www.google.com/",
"id": "AbC123"
}
}


### Public URL Shortening (No API Key Required)

POST /api/public/shorten
Content-Type: text/plain

Example
curl -d 'https://www.example.com/'
http://localhost:8000/api/public/shorten


**Response**: Same JSON format as authenticated endpoint, but may have stricter rate limits.

### Redirect to Original URL

GET /api/redirect/{id}

Example
curl -L http://localhost:8000/api/redirect/AbC123


**Response**: HTTP 308 Permanent Redirect to the original URL

### Short Redirect (Root Path)

GET /{id}

Example - Cleaner URL format
curl -L http://localhost:8000/AbC123


**Response**: HTTP 308 Permanent Redirect to the original URL

**Note**: This is an alternative to `/api/redirect/{id}` for cleaner URLs.

### Health Check

GET /api/health_check

Example
curl http://localhost:8000/api/health_check


**Response**: HTTP 200 OK with JSON envelope

{
"success": true,
"message": "ok",
"status": 200,
"time": "2025-09-18T12:00:00Z",
"data": null
}


### Admin Interface

GET /admin

Example - View the web interface
curl http://localhost:8000/admin


**Response**: HTML page with admin interface

**Additional Admin Routes** (all require API key):
- `GET /admin/profile` - User profile management
- `GET /admin/login` - Login page
- `GET /admin/register` - Registration page

For complete route documentation, see [ROUTE_ORGANIZATION.md](ROUTE_ORGANIZATION.md).

## 📖 API Documentation

The URL Shortener service provides comprehensive API documentation with OpenAPI 3.0 specification and interactive Swagger UI.

### Interactive API Documentation

Visit the Swagger UI at: `http://localhost:8000/api/docs`

The interactive documentation provides:

- **Complete API reference** with all endpoints, parameters, and responses
- **Interactive testing** - Try out API calls directly from your browser
- **Request/response examples** for all endpoints
- **Authentication support** for protected endpoints
- **Schema validation** with automatic request/response validation

### OpenAPI Specification

The OpenAPI 3.0 specification is available at: `http://localhost:8000/api/docs/openapi.yaml`

This YAML file can be used with:

- **API clients** like Postman, Insomnia, or REST Client
- **Code generation tools** to generate client SDKs
- **Documentation tools** for custom API documentation sites
- **API testing tools** for automated testing

### API Features Documented

- **URL Shortening**: Create short URLs with optional custom aliases
- **URL Redirection**: Fast redirects to original URLs
- **Health Monitoring**: Service health checks
- **Authentication**: API key-based authentication for protected endpoints
- **Rate Limiting**: Built-in rate limiting information
- **Error Handling**: Comprehensive error response documentation

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [SQLx CLI](https://crates.io/crates/sqlx-cli)
- Database: SQLite (no setup required) or PostgreSQL (optional)
- [Nix](https://nixos.org/download.html) (optional, for Nix development environment)

### Local Development

#### Option 1: Traditional Rust Development

1. **Clone the repository**

git clone https://github.com/zero-to-mastery/url-shortener-ztm.git
cd url-shortener-ztm


2. **Install dependencies**

cargo build


3. **Create the Database**

sqlx database create


4. **Run the application**

cargo run


The database and migrations will be set up automatically on first run.

#### Option 2: Nix Development Environment

1. **Clone the repository**

git clone https://github.com/zero-to-mastery/url-shortener-ztm.git
cd url-shortener-ztm


2. **Enter the Nix development environment**

nix develop --accept-flake-config # --accept-flake-config is needed to accept the nix-community binary cache for faster builds.


This provides a complete development environment with Rust toolchain, SQLx CLI, and all dependencies.

3. **Run the application**

cargo run


4. **Test the service**

Get your API key from configuration/base.yml (or set via environment)
API_KEY="e4125dd1-3d3e-43a1-bc9c-dc0ba12ad4b5"

Shorten a URL
curl -d 'https://example.com'
-H "x-api-key: $API_KEY"
http://localhost:8000/api/shorten

Visit the shortened URL
curl -L http://localhost:8000/AbC123

Check health
curl http://localhost:8000/api/health_check

Visit admin interface
open http://localhost:8000/admin


### Using Just Command Runner

This project uses [just](https://github.com/casey/just) as a command runner for common development tasks. Think of it like `make` but simpler and more user-friendly.

**Available Commands:**

List all available commands
just --list

Start in release mode (rate limiting ON, log level: warn)
just start

Start in development mode (rate limiting OFF, log level: debug)
just start-dev

Start with custom settings
just start rate="false" log="info"
just start-dev rate="true" log="trace"

Prepare test data
just prepare-shorten-data
just prepare-redirect-data

Run performance tests
just perf-shorten # Test URL shortening performance
just perf-redirect # Test redirect performance
just perf-shorten-bench # Run benchmark suite


**Installing Just:**

macOS
brew install just

Linux
cargo install just

Windows
scoop install just


**Note**: This project uses **Nushell** as the shell for `just` commands. Install Nushell from [nushell.sh](https://www.nushell.sh/) if you want to run performance tests.

For more information, visit the [Just documentation](https://just.systems).

### Configuration

The application supports environment-based configuration with YAML files:

#### Configuration Files

- `configuration/base.yml` - Base configuration (application, database, rate limiting)
- `configuration/generator.yml` - ID generator configuration (nanoid/sequence settings)
- `configuration/local.yml` - Local development overrides
- `configuration/production.yml` - Production settings

#### Environment Variables

Set `APP_ENVIRONMENT` to `local` or `production` to load the appropriate config.

Override any setting using environment variables with `APP_` prefix. **Note**: Use double underscores (`__`) to access nested configuration values:

Application settings
APP_APPLICATION__PORT=3000
APP_APPLICATION__HOST=0.0.0.0
APP_APPLICATION__API_KEY=your-new-api-key

Database settings
APP_DATABASE__TYPE=sqlite # or "postgres"
APP_DATABASE__URL=sqlite:database.db
APP_DATABASE__CREATE_IF_MISSING=true

Rate limiting
APP_RATE_LIMITING__ENABLED=false
APP_RATE_LIMITING__REQUESTS_PER_SECOND=100
APP_RATE_LIMITING__BURST_SIZE=20


**Configuration Hierarchy:**
- `APP_` prefix indicates environment variable
- Double underscore (`__`) separates nested YAML keys
- Example: `APP_DATABASE__URL` maps to `database.url` in YAML
- Example: `APP_APPLICATION__API_KEY` maps to `application.api_key` in YAML

**Generator Configuration:**

The `configuration/generator.yml` file controls ID generation behavior:

shortener:
length: 7 # Length of generated short codes
alphabet: "0-9A-Za-z" # Characters used in short codes
engine:
kind: "nanoid" # Generator type: "nanoid" or "sequence"


Override via environment:
APP_SHORTENER__LENGTH=8
APP_SHORTENER__ENGINE__KIND=sequence


### API Key Security

The service protects write endpoints with a UUID-based API key.

- The base config includes an **obviously insecure development key** so `cargo run` works out of the box.
- On startup, the app detects this default key and prints a prominent warning to the console.
- In any non-local environment, you MUST override the key via environment variable.

Generate a UUID v4:

Linux/macOS
uuidgen

PowerShell
Rust (optional helper):
cargo run --bin print-uuid


Set the key via env var:

APP_APPLICATION__API_KEY=$(uuidgen)


Production guidance:

- Store secrets in your platform's secret manager (e.g., Fly.io, Railway, Kubernetes, GitHub Actions).
- Rotate keys when compromised or on developer offboarding.
- Never commit real keys to version control.

#### Database Configuration

**SQLite Configuration (Default)**

database:
type: sqlite
url: "sqlite:database.db" # Path to SQLite database file
create_if_missing: true # Create database if it doesn't exist
max_connections: 16 # optional set database pool connection
min_connections: 4 # optional set database pool connection


**PostgreSQL Configuration**

database:
type: postgres
host: "localhost"
port: 5432
username: "app"
password: "secret"
database_name: "urlshortener"
max_connections: 64 # optional set database pool connection
min_connections: 16 # optional set database pool connection
create_if_missing: true


**For in-memory database (testing):**

database:
type: sqlite
url: ":memory:"
create_if_missing: true


#### Rate Limiting Configuration

The service includes built-in rate limiting to prevent abuse using the [tower-governor](https://crates.io/crates/tower_governor) crate:

rate_limiting:
enabled: true # Enable/disable rate limiting
requests_per_second: 10 # Maximum sustained request rate per IP
burst_size: 5 # Additional burst capacity per IP


**Environment-specific examples:**

**Development** (`configuration/local.yml`):

rate_limiting:
enabled: true
requests_per_second: 20 # More lenient for development
burst_size: 10


**Production** (`configuration/production.yml`):

rate_limiting:
enabled: true
requests_per_second: 5 # Strict rate limiting for production
burst_size: 3


**Rate Limiting Behavior:**

- Limits are applied **per IP address** using the GCRA (Generic Cell Rate Algorithm)
- Only **URL shortening endpoints** are rate limited (`/api/shorten`, `/api/public/shorten`)
- Health checks and redirects are **not rate limited**
- Standard HTTP headers are included in rate limit responses:
  - `retry-after`: Seconds to wait before retrying
  - `x-ratelimit-after`: Additional rate limiting information
- Returns **HTTP 429 Too Many Requests** when limits are exceeded

**Environment Variable Override:**

APP_RATE_LIMITING__ENABLED=false # Disable rate limiting
APP_RATE_LIMITING__REQUESTS_PER_SECOND=100 # 100 requests per second
APP_RATE_LIMITING__BURST_SIZE=20 # Allow bursts of 20 requests


## 🧪 Testing

The project includes comprehensive integration tests using in-memory databases.

Run all tests
cargo test

Run tests with logging output
TEST_LOG=1 cargo test

Run specific test module
cargo test health_check
cargo test redirect
cargo test shorten

Run PostgreSQL tests (requires running PostgreSQL)
cargo test postgres_database_insert_get -- --ignored


### Test Coverage

- ✅ Health check endpoint with JSON envelope validation
- ✅ URL shortening functionality with API key authentication
- ✅ URL redirection with proper HTTP status codes
- ✅ URL length validation (2048 character limit)
- ✅ Rate limiting with per-IP enforcement and proper HTTP headers
- ✅ SQLite database integration with trait abstraction
- ✅ PostgreSQL database integration (optional)
- ✅ Error handling and edge cases

## 🏗 Project Structure

src/
├── bin/
│ └── main.rs # Application entry point
├── lib.rs # Library crate root
├── configuration.rs # Configuration management
├── errors.rs # Error types and handling
├── middleware.rs # API key authentication
├── response.rs # JSON response envelope
├── startup.rs # Application startup and router
├── state.rs # Application state management
├── telemetry.rs # Logging and tracing setup
├── templates.rs # Template rendering
├── database/
│ ├── mod.rs # Database trait definitions
│ ├── sqlite.rs # SQLite implementation
│ └── postgres_sql.rs # PostgreSQL implementation
├── generator/
│ ├── mod.rs # ID generator module
│ ├── config.rs # Generator configuration
│ ├── nanoid.rs # Nanoid generator implementation
│ └── sequence.rs # Sequential ID generator
├── models/
│ └── mod.rs # Data models
├── shortcode/
│ ├── mod.rs # Short code management
│ └── bloom_filter.rs # Bloom filter for collision detection
└── routes/
├── mod.rs # Route module exports
├── health_check.rs # Health check handler
├── index.rs # Index page handler
├── admin.rs # Admin interface handler
├── docs.rs # API documentation (Swagger/OpenAPI)
├── shorten.rs # URL shortening handler
└── redirect.rs # URL redirect handler

tests/
├── api/
│ ├── main.rs # Integration test entry
│ ├── helpers.rs # Test utilities and setup
│ ├── health_check.rs # Health check tests
│ ├── shorten.rs # URL shortening tests
│ ├── redirect.rs # URL redirect tests
│ ├── rate_limiting.rs # Rate limiting tests
│ ├── error_handling.rs # Error handling tests
│ ├── alias_validation_consistency.rs # Alias validation tests
│ └── static_assets.rs # Static asset serving tests
└── perf/
├── shorten.js # Performance tests for shortening
├── redirect.js # Performance tests for redirects
├── shortener-bench.js # Benchmark suite
└── run_shortener-bench.nu # Benchmark runner script

configuration/
├── base.yml # Base configuration
├── generator.yml # ID generator configuration
├── local.yml # Local development config
└── production.yml # Production config

migrations/
├── 20251017163705_url_shortener_ztm.up.sql # SQLite schema
├── 20251017163705_url_shortener_ztm.down.sql # SQLite rollback
├── 20251017184220_add_users_and_sessions.up.sql # SQLite users/sessions
├── 20251017184220_add_users_and_sessions.down.sql # SQLite users/sessions rollback
├── 20251107120000_add_bloom_snapshots_table.up.sql # SQLite bloom filter snapshots
├── 20251107120000_add_bloom_snapshots_table.down.sql # SQLite bloom filter rollback
└── pg/ # PostgreSQL migrations
├── 20251015003911_url_shortener_ztm_pg.up.sql # PostgreSQL schema
├── 20251015003911_url_shortener_ztm_pg.down.sql # PostgreSQL rollback
├── 20251015102402_init_url_shortener.up.sql # PostgreSQL initialization
├── 20251015102402_init_url_shortener.down.sql # PostgreSQL init rollback
├── 20251107120000_add_bloom_snapshots_table.up.sql # PostgreSQL bloom filter
└── 20251107120000_add_bloom_snapshots_table.down.sql # PostgreSQL bloom rollback

scripts/ # Utility scripts (Nushell)
├── gen_repeat_url.nu # Generate repeated URLs for testing
├── gen_req_url.nu # Generate request URLs
├── gen_short_id.nu # Generate short IDs
├── get_ulr_data_from_db.nu # Query URL data from database
├── get_urls.nu # Fetch URLs
├── helpers.nu # Helper functions
├── prepare_redirect_data.nu # Prepare redirect test data
└── prepare_shorten_data.nu # Prepare shorten test data

static/ # Static web assets
├── screen.css # CSS styles
└── scripts.js # JavaScript

templates/ # Tera templates
├── base.html # Base template
├── index.html # Index page
├── admin.html # Admin interface
├── login.html # Login page
├── profile.html # User profile page
└── register.html # Registration page

docs/
└── deployment-guide.md # Deployment documentation

justfile # Just command runner recipes
openapi.yaml # OpenAPI 3.0 specification
flake.nix # Nix development environment
flake.lock # Nix lock file


**For detailed route organization**, see [ROUTE_ORGANIZATION.md](ROUTE_ORGANIZATION.md).

## 🔧 Architecture

### Database Layer

The application uses a trait-based database abstraction (`UrlDatabase`) that supports both SQLite and PostgreSQL:

#[async_trait]
pub trait UrlDatabase: Send + Sync {
async fn insert_url(&self, id: &str, url: &str) -> Result<(), DatabaseError>;
async fn get_url(&self, id: &str) -> Result<String, DatabaseError>;
}


### Error Handling

Comprehensive error handling with custom `ApiError` types and structured JSON responses:

pub enum ApiError {
BadRequest(String),
NotFound(String),
Unauthorized(String),
Internal(String),
// ...
}


### Configuration Management

Layered configuration system supporting YAML files and environment variables with automatic environment detection.

## 📊 Database Schema

### URLs Table

CREATE TABLE urls (
id INTEGER PRIMARY KEY,
code TEXT NOT NULL UNIQUE, -- Short identifier (nanoid, 7 characters)
url TEXT NOT NULL, -- Original URL
url_hash BLOB NOT NULL UNIQUE -- SHA-256 hash for deduplication
);


### Aliases Table

CREATE TABLE aliases (
alias TEXT PRIMARY KEY, -- Custom alias for a URL
target_id INTEGER NOT NULL REFERENCES urls(id) ON DELETE CASCADE
);

CREATE INDEX aliases_target_id_idx ON aliases(target_id);


### All Short Codes View

CREATE VIEW all_short_codes AS
SELECT u.code AS code, u.id AS target_id, u.url AS url, 'code' AS source
FROM urls u
UNION ALL
SELECT a.alias AS code, a.target_id, u.url, 'alias' AS source
FROM aliases a
JOIN urls u ON u.id = a.target_id;


This view combines primary codes and aliases for unified lookups.

### Bloom Filter Snapshots Table

CREATE TABLE bloom_snapshots (
name TEXT PRIMARY KEY,
data BLOB NOT NULL, -- Serialized bloom filter
updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);


### Key Features

- **`url_hash`** - SHA-256 hash enables deduplication of identical URLs
- **`aliases`** - Supports custom short code aliases for memorable URLs
- **`all_short_codes`** - View provides unified access to both codes and aliases
- **`bloom_snapshots`** - Persists bloom filter for fast collision detection across restarts
- **Triggers** - Database triggers prevent conflicts between codes and aliases

## 🔍 Monitoring & Observability

- **Structured Logging**: JSON-formatted logs with request correlation IDs
- **Request Tracing**: Full request lifecycle tracing with `tracing` crate
- **Health Checks**: `/api/health_check` endpoint with JSON envelope response
- **Error Handling**: Comprehensive error responses with appropriate HTTP status codes
- **Request IDs**: Automatic request ID generation and propagation

## 🔒 Security

- **API Key Authentication**: Protected endpoints require valid UUID-based API keys
- **Input Validation**: URL parsing and length validation before storage
- **SQL Injection Protection**: Type-safe queries with SQLx
- **Error Information Disclosure**: Sanitized error responses
- **Resource Protection**: URL length limits prevent resource exhaustion attacks

## 🚧 Roadmap

- [x] SQLite database support with migrations
- [x] Database abstraction layer
- [x] Web UI with Tera templates
- [x] API key authentication
- [x] Comprehensive error handling
- [x] Integration tests
- [x] PostgreSQL database support
- [x] Rate limiting with tower-governor
- [x] URL length validation (2048 characters)
- [x] Nix development environment with flake
- [ ] User authentication and URL management
- [ ] Analytics and usage statistics
- [ ] Custom short URL aliases
- [ ] URL expiration and cleanup
- [ ] Docker containerization
- [ ] Real-world API specification compliance

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Ensure all tests pass (`cargo test`)
- Follow Rust naming conventions
- Add tests for new functionality
- Update documentation as needed

## 📄 License

This project is licensed under the MIT License - see the [License.txt](License.txt) file for details.

## 👤 Author

**Jeffery D. Mitchell**

- Email: [crusty.rustacean@gmail.com](mailto:crusty.rustacean@gmail.com)
- GitHub: [@crustyrustacean](https://github.com/crustyrustacean)

## 🙏 Acknowledgments

- Built with the excellent Rust web ecosystem
- Inspired by modern web service architecture patterns
- Thanks to the Rust community for amazing tools and libraries

## 🛠️ Deployment Guide

You can deploy this project to various platforms like Railway, Fly.io, and DigitalOcean.

👉 Check the full [Deployment Guide](docs/deployment-guide.md) for detailed instructions.