# Architecture Overview

This document provides a comprehensive overview of the URL Shortener application architecture, designed to help contributors understand the system structure and design decisions.

## 🏗️ High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              HTTP Layer                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌───────────┐ │
│  │    Static    │    │   Admin UI   │    │     API      │    │  Health   │ │
│  │    Files     │    │   /admin     │    │  Endpoints   │    │   Check   │ │
│  │              │    │              │    │              │    │           │ │
│  └──────────────┘    └──────────────┘    └──────────────┘    └───────────┘ │
│                                                  │                          │
├─────────────────────────────────────────────────┼──────────────────────────┤
│                         Middleware Layer        │                          │
│                                                  │                          │
│  ┌────────────────────┐ ┌────────────────────────▼─────────────────────────┐│
│  │   Rate Limiting    │ │         API Key Authentication                   ││
│  │  (tower-governor)  │ │         (Protected Routes Only)                  ││
│  └────────────────────┘ └────────────────────────┬─────────────────────────┘│
├──────────────────────────────────────────────────┼──────────────────────────┤
│                       Application Layer          │                          │
│                                                   │                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────▼─────────────┐            │
│  │   URL Redirect  │  │  URL Shortening │  │   Template        │            │
│  │    Handler      │  │     Handler     │  │   Rendering       │            │
│  │                 │  │                 │  │                   │            │
│  └─────────────────┘  └─────────────────┘  └───────────────────┘            │
│           │                     │                     │                     │
├───────────┼─────────────────────┼─────────────────────┼─────────────────────┤
│           │                     │                     │                     │
│           ▼                     ▼                     ▼                     │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                     Application State                                │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────────┐│  │
│  │  │  Database   │  │   API Key   │  │       Template Directory        ││  │
│  │  │   Handle    │  │             │  │                                 ││  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────────┘│  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│           │                                                                 │
├───────────┼─────────────────────────────────────────────────────────────────┤
│           ▼                                                                 │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                      Database Layer                                  │  │
│  │                                                                      │  │
│  │  ┌─────────────────┐              ┌─────────────────────────────────┐ │  │
│  │  │  UrlDatabase    │◄─────────────┤      SQLite Implementation     │ │  │
│  │  │     Trait       │              │                                 │ │  │
│  │  │                 │              │  ┌─────────────────────────────┐│ │  │
│  │  │ - insert_url()  │              │  │     Connection Pool        ││ │  │
│  │  │ - get_url()     │              │  │                             ││ │  │
│  │  │                 │              │  └─────────────────────────────┘│ │  │
│  │  │                 │              └─────────────────────────────────┘ │  │
│  │  │                 │                                                   │  │
│  │  │                 │              ┌─────────────────────────────────┐ │  │
│  │  │                 │◄─────────────┤    PostgreSQL Implementation   │ │  │
│  │  │                 │              │                                 │ │  │
│  │  │                 │              │  ┌─────────────────────────────┐│ │  │
│  │  │                 │              │  │     Connection Pool        ││ │  │
│  │  │                 │              │  │                             ││ │  │
│  │  └─────────────────┘              │  └─────────────────────────────┘│ │  │
│  │                                   └─────────────────────────────────┘ │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                           │                                 │
└───────────────────────────────────────────┼─────────────────────────────────┘
                                            ▼
                                   ┌─────────────────┐
                                   │  SQLite File    │
                                   │   database.db   │
                                   │       OR        │
                                   │ PostgreSQL DB   │
                                   └─────────────────┘
```

## 🔧 Core Components

### 1. Application Entry Point (`src/bin/main.rs`)

The main entry point orchestrates the application startup:

- **Telemetry Initialization**: Sets up structured logging with tracing
- **Configuration Loading**: Reads YAML configs and environment variables
- **Application Bootstrap**: Creates and starts the web server
- **Graceful Shutdown**: Handles SIGTERM/SIGINT signals

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging → Load config → Build app → Run until stopped
}
```

### 2. Configuration System (`src/lib/configuration.rs`)

Multi-layered configuration management using Figment:

- **Base Configuration**: `configuration/base.yml`
- **Environment Overrides**: `configuration/local.yml` or `configuration/production.yml`
- **Environment Variables**: `APP_*` prefixed variables
- **Type Safety**: Strongly typed configuration structs with validation

**Configuration Hierarchy**: Environment Variables > Environment File > Base File

### 3. Database Layer (`src/lib/database/`)

**Design Pattern**: Repository Pattern with Trait Abstraction

```rust
#[async_trait]
pub trait UrlDatabase: Send + Sync {
    async fn insert_url(&self, id: &str, url: &str) -> Result<(), DatabaseError>;
    async fn get_url(&self, id: &str) -> Result<String, DatabaseError>;
}
```

**Benefits**:
- **Testability**: Easy to mock for unit tests
- **Flexibility**: Can swap database implementations (PostgreSQL, MySQL, etc.)
- **Type Safety**: SQLx provides compile-time SQL validation

**Current Implementations**:
- **SQLite**: File-based database with connection pooling and automatic migrations
- **PostgreSQL**: Networked database with connection pooling and dedicated migrations

### 4. HTTP Layer (`src/lib/routes/`)

RESTful API design with clear separation of concerns:

#### Endpoints:

- **`POST /api/shorten`** (Protected) - Create shortened URLs
- **`GET /api/redirect/{id}`** - Redirect to original URL
- **`GET /api/health_check`** - Service health status
- **`GET /admin`** - Admin web interface
- **Static Files** - CSS/JS assets

#### Route Handlers:
Each handler follows a consistent pattern:
1. Extract request data (path params, body, headers)
2. Validate input (URL format, length limits, security checks)
3. Interact with database layer
4. Return structured response or error

**Input Validation Features**:
- **URL Length Validation**: Configurable maximum URL length (default: 2048 characters)
- **URL Format Validation**: RFC-compliant URL parsing using `url` crate
- **Security Validation**: Prevents malicious input and resource exhaustion
- **Early Validation**: Length checks before expensive parsing operations

### 5. Middleware Stack (`src/lib/middleware.rs`)

**Request Processing Pipeline**:
```
Request → Request ID → Tracing → Rate Limiting → API Key Auth → Handler → Response
```

- **Request ID Generation**: UUID-based correlation IDs
- **Distributed Tracing**: Request lifecycle tracking
- **Rate Limiting**: Per-IP rate limiting using GCRA algorithm (tower-governor)
- **API Key Authentication**: Protects sensitive endpoints
- **Error Handling**: Converts errors to HTTP responses

**Rate Limiting Features**:
- **Per-IP Enforcement**: Individual rate limits for each client IP
- **GCRA Algorithm**: Generic Cell Rate Algorithm for smooth rate limiting
- **Configurable**: Requests per second and burst size configurable
- **Selective**: Only applies to URL shortening endpoints, not redirects/health checks

### 6. Application State (`src/lib/state.rs`)

**Shared State Pattern**: Dependency injection container

```rust
pub struct AppState {
    pub database: Arc<dyn UrlDatabase>,
    pub api_key: Uuid,
    pub template_dir: String,
}
```

**Benefits**:
- **Thread Safety**: Arc allows sharing across async tasks
- **Dependency Injection**: Handlers receive dependencies through state
- **Configuration**: Runtime configuration accessible to all handlers

### 7. Error Handling (`src/lib/errors.rs`)

**Centralized Error Management**:

```rust
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    // ... other variants
}
```

**Features**:
- **HTTP Status Mapping**: Errors automatically convert to appropriate HTTP codes
- **Error Chain Preservation**: Full error context maintained for debugging
- **Consistent API Responses**: All errors return JSON envelope format

### 8. Response Format (`src/lib/response.rs`)

**JSON Envelope Pattern**:

```json
{
  "success": true|false,
  "message": "descriptive message",
  "status": 200,
  "time": "2025-09-23T12:00:00Z",
  "data": { ... } | null
}
```

**Benefits**:
- **Consistent Structure**: All API responses follow same format
- **Client-Friendly**: Easy to parse and handle on frontend
- **Debugging**: Timestamps and messages aid troubleshooting

### 9. Template Engine (`src/lib/templates.rs`)

**Tera Integration**:
- **Template Compilation**: Templates compiled once at startup
- **Inheritance**: Base templates with block extensions
- **Context Injection**: Dynamic data binding
- **Static Assets**: CSS/JS served from `/static`

## 🔄 Request Flow

### URL Shortening Flow
```
1. POST /api/shorten with URL in body
2. Rate limiting middleware checks IP-based limits
3. API Key middleware validates x-api-key header
4. Handler validates URL length (max 2048 characters)
5. Handler parses and validates URL format
6. Generate unique 6-character ID (nanoid)
7. Store mapping in database
8. Return shortened URL: https://host/{id}
```

### URL Redirect Flow
```
1. GET /api/redirect/{id}
2. Handler extracts ID from path
3. Database lookup for original URL
4. Return HTTP 308 Permanent Redirect
5. Browser follows redirect to original URL
```

### Admin Interface Flow
```
1. GET /admin
2. Handler loads template context
3. Tera renders HTML with base template
4. Static CSS/JS loaded from /static
5. Return complete HTML page
```

## 📊 Data Model

### Database Schema

**Core URLs Table**:
```sql
CREATE TABLE urls (
    id TEXT PRIMARY KEY,         -- 6-character nanoid
    url TEXT NOT NULL,           -- Original URL
    owner_id TEXT NULL           -- FK to users.id (optional)
);
```

**User Management Tables**:
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,                    -- UUID for portability
    email TEXT NOT NULL UNIQUE,            -- User email address
    password_hash TEXT NOT NULL,           -- Secure password hash
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,                    -- Session UUID
    user_id TEXT NOT NULL,                 -- FK to users.id
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,          -- Session expiry
    last_active_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_agent TEXT,                       -- Optional metadata
    ip_address TEXT,                       -- Optional metadata
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

**Design Decisions**:
- **Text IDs**: Human-readable, URL-safe identifiers across all tables
- **Optional User Association**: URLs can be anonymous or user-owned
- **Session Management**: Foundation for user authentication system
- **Backward Compatibility**: Existing anonymous URLs remain functional
- **Performance Indexes**: Strategic indexes for common lookup patterns

## 🏗️ Architectural Patterns

### 1. Layered Architecture
- **Presentation Layer**: HTTP handlers and templates
- **Business Logic Layer**: URL shortening logic
- **Data Access Layer**: Database abstraction
- **Infrastructure Layer**: Configuration, logging, startup

### 2. Dependency Injection
- **State Container**: AppState provides dependencies to handlers
- **Interface Segregation**: Small, focused traits
- **Inversion of Control**: Handlers depend on abstractions, not implementations

### 3. Repository Pattern
- **Data Access Abstraction**: UrlDatabase trait
- **Implementation Flexibility**: Easy to swap database types
- **Testing**: Mock implementations for unit tests

### 4. Command Query Separation
- **Commands**: `insert_url()` modifies state
- **Queries**: `get_url()` reads state
- **Clear Intent**: Method names indicate read vs write operations

## 🧪 Testing Architecture

### Integration Tests (`tests/api/`)

**Test Strategy**:
- **In-Memory Database**: SQLite `:memory:` for fast, isolated tests
- **PostgreSQL Integration**: Optional tests against real PostgreSQL instances
- **Full Application**: Tests complete request/response cycle
- **Helper Functions**: Shared test utilities and setup
- **Tracing Integration**: Optional logging for debugging
- **Multi-Database**: Tests run against multiple database backends

**Test Structure**:
```
tests/api/
├── main.rs          # Test module declarations
├── helpers.rs       # TestApp and spawn_app()
├── health_check.rs  # Health endpoint tests
├── shorten.rs       # URL shortening tests
└── redirect.rs      # URL redirect tests
```

### Test Application Setup
```rust
pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    pub database: Arc<dyn UrlDatabase>,
    pub api_key: Uuid,
}
```

## 🚀 Deployment Considerations

### Configuration Management
- **Environment-Based**: Different configs for local/production
- **Environment Variables**: Override any setting via `APP_*` variables
- **Secrets Management**: API keys and sensitive data via env vars

### Database
- **File-Based**: SQLite database stored as `database.db`
- **Migrations**: Automatic migration on startup
- **Backup**: Simple file backup/restore

### Monitoring
- **Health Checks**: `/api/health_check` for load balancer probes
- **Structured Logging**: JSON logs for log aggregation
- **Request Tracing**: Correlation IDs for debugging
- **Error Handling**: Comprehensive error responses

## 🔮 Extension Points

### Database Support
The trait-based database abstraction currently supports multiple databases:

**Implemented**:
- **SQLite**: `SqliteUrlDatabase` in `src/database/sqlite.rs`
- **PostgreSQL**: `PostgresUrlDatabase` in `src/database/postgres_sql.rs`

**Adding New Databases**:
```rust
pub struct MySqlUrlDatabase { /* ... */ }

impl UrlDatabase for MySqlUrlDatabase {
    // Implement trait methods for MySQL
}
```

**Database Selection**: Configured through environment settings, enabling easy switching between database backends.

### Authentication
Current API key middleware can be extended for:
- User-based authentication
- JWT tokens  
- OAuth integration
- Rate limiting per user

### Features
The modular architecture supports adding:
- Custom short URL aliases
- URL expiration dates
- Analytics and click tracking
- User dashboards
- Bulk URL operations

### Caching
Add caching layer between handlers and database:
- Redis for distributed caching
- In-memory cache for single instance
- Cache invalidation strategies

## 🛠️ Development Environment

### Nix Flake Integration

The project includes a comprehensive Nix development environment (`flake.nix`):

**Features**:
- **Reproducible Environment**: Consistent development setup across all platforms
- **Rust Toolchain**: Fenix-provided stable Rust with required components
- **Dependencies**: Pre-installed SQLx CLI, SQLite, and development tools
- **LLVM Integration**: Clang/LLD for optimized linking
- **Pre-commit Hooks**: Optional formatting and linting checks

**Usage**:
```bash
# Enter development environment
nix develop --accept-flake-config # --accept-flake-config is needed to accept the nix-community binary cache for faster builds.

# Automatic environment with direnv
echo "use flake . --accept-flake-config" > .envrc
direnv allow
```

**Benefits**:
- **Zero Setup**: No manual dependency installation
- **Version Consistency**: Locked dependency versions across team
- **CI/CD Integration**: Same environment used in continuous integration
- **Cross-Platform**: Works on Linux, macOS, and Windows (WSL)

## 📝 Development Guidelines

### Adding New Endpoints
1. Create handler in `src/lib/routes/`
2. Add route to router in `src/lib/startup.rs`
3. Add integration tests in `tests/api/`
4. Update API documentation

### Database Changes
1. Create migration files in `migrations/` (SQLite) and `migrations/pg/` (PostgreSQL)
2. Update database trait if needed (`src/database/mod.rs`)
3. Update implementations (`src/database/sqlite.rs` and `src/database/postgres_sql.rs`)
4. Test migrations on both database backends
5. Update configuration examples for new settings

### Configuration Changes
1. Update `Settings` structs in `configuration.rs`
2. Add to relevant YAML files
3. Update documentation
4. Add validation if needed

---

This architecture provides a solid foundation for a URL shortener service while remaining extensible and maintainable. The clear separation of concerns and trait-based abstractions make it easy for contributors to understand and extend the system.