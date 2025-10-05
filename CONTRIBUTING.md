# Contributing to URL Shortener

We welcome contributions to the URL Shortener project! This guide will help you get started with contributing to this Rust-based web service.

## 🎃 Hacktoberfest 2025

This project is participating in Hacktoberfest 2025! We welcome contributions from developers of all skill levels.

### Issue Assignment Process

**Important**: Comment on the issue to have it assigned to you, DO NOT start working the issue until it has been assigned to you. To respect the time and effort of students that took the time to request the issue assignment, PRs created by someone other than the assigned will be closed.

## 🚀 Quick Start

### Prerequisites

Before contributing, make sure you have:

- [Rust](https://rustup.rs/) (latest stable version)
- [SQLx CLI](https://crates.io/crates/sqlx-cli) for database operations
- Git for version control
- A text editor or IDE (VS Code with rust-analyzer recommended)
- **Optional**: [Nix](https://nixos.org/download.html) for reproducible development environment
- **Optional**: PostgreSQL (if working on PostgreSQL-specific features)

### Setting Up Your Development Environment

#### Option 1: Traditional Rust Setup

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/url-shortener-ztm.git
   cd url-shortener-ztm
   ```

2. **Install dependencies**
   ```bash
   cargo build
   ```

3. **Set up the database**
   ```bash
   sqlx database create
   ```

4. **Run the application**
   ```bash
   cargo run
   ```

   The application will be available at `http://localhost:8000`

5. **Run tests to ensure everything works**
   ```bash
   cargo test
   ```

#### Option 2: Nix Development Environment (Recommended)
- If you don’t have Nix yet, you can use the Determinate installer to install nix and setup `nix flake` support be default:
```
   curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install --determinate`
```

For a consistent, reproducible development environment:

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/url-shortener-ztm.git
   cd url-shortener-ztm
   ```

2. **Enter the Nix development environment**
   ```bash
    nix develop --accept-flake-config # --accept-flake-config is needed to accept the nix-community binary cache for faster builds.
   ```

   This automatically provides:
   - Rust toolchain with required components
   - SQLx CLI and SQLite
   - Development tools and dependencies
   - Consistent environment across all platforms

3. **Run the application**
   ```bash
   cargo run
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

#### Option 3: Using direnv (Automatic Environment)

For automatic environment activation:

1. **Install direnv** (if not already installed)

> Enable nix-direnv Follow: https://nix.dev/guides/recipes/direnv.html

2. **Create .envrc file**
   ```bash
   echo "use flake . --accept-flake-config" > .envrc
   direnv allow
   ```
3. **Environment loads automatically** when you `cd` into the project

## 🤝 How to Contribute

### Types of Contributions Welcome

- 🐛 **Bug fixes** - Fix issues in the codebase
- ✨ **New features** - Add functionality to the URL shortener
- 📚 **Documentation** - Improve README, code comments, or create tutorials
- 🧪 **Tests** - Add or improve test coverage (SQLite and PostgreSQL)
- 🎨 **UI/UX improvements** - Enhance the web interface
- ⚡ **Performance optimizations** - Make the service faster or more efficient
- 🔒 **Security enhancements** - Improve security, rate limiting, input validation
- 🏗️ **Refactoring** - Improve code structure and maintainability
- 🗄️ **Database improvements** - Enhance SQLite/PostgreSQL implementations
- 🔧 **Development environment** - Improve Nix flake or development tooling
- 📊 **Rate limiting** - Enhance or customize rate limiting features
- ✅ **Input validation** - Improve URL validation and security checks

### Finding Issues to Work On

- Look for issues labeled `hacktoberfest`, `good first issue`, or `help wanted`
- Check the project's GitHub Issues tab
- Issues labeled `beginner-friendly` are great for new contributors

## 📝 Development Guidelines

### Code Style

- **Format your code**: Run `cargo fmt` before committing
- **Lint your code**: Run `cargo clippy` and fix any warnings
- **Follow Rust conventions**: Use snake_case for functions and variables, PascalCase for types
- **Write descriptive commit messages**: Use conventional commit format when possible

### Testing

- **Write tests for new features**: Add both unit and integration tests
- **Ensure all tests pass**: Run `cargo test` before submitting a PR
- **Test edge cases**: Consider error conditions and boundary cases
- **Use the test helpers**: Utilize the existing test infrastructure in `tests/api/helpers.rs`

#### Running Tests

```bash
# Run all tests (uses in-memory SQLite by default)
cargo test

# Run tests with logging output (useful for debugging)
TEST_LOG=1 cargo test

# Run specific test modules
cargo test health_check
cargo test shorten
cargo test redirect

# Run PostgreSQL integration tests (requires running PostgreSQL)
cargo test postgres_database_insert_get -- --ignored

# Run tests with specific database configuration
DATABASE_URL="sqlite::memory:" cargo test
```

#### Testing Multiple Database Backends

The project supports both SQLite and PostgreSQL. When contributing:

- **SQLite tests** run automatically (in-memory database)
- **PostgreSQL tests** are marked with `#[ignore]` and require a running PostgreSQL instance
- Test your changes against both backends when working on database-related features
- Use the test helpers in `tests/api/helpers.rs` for consistent test setup

### Database Changes

If your contribution involves database changes:

#### SQLite Migrations
1. Create migration files in the `migrations/` directory
2. Follow the naming convention: `YYYYMMDDHHMMSS_description.up.sql` and `.down.sql`
3. Test your migration with `sqlx migrate run` and `sqlx migrate revert`

#### PostgreSQL Migrations
1. Create migration files in the `migrations/pg/` directory
2. Use the same naming convention as SQLite migrations
3. Test migrations against a running PostgreSQL instance
4. Ensure both SQLite and PostgreSQL migrations achieve the same schema result

#### Database Implementation Changes
- Update the `UrlDatabase` trait in `src/database/mod.rs` if adding new methods
- Implement changes in both `src/database/sqlite.rs` and `src/database/postgres_sql.rs`
- Add appropriate error handling for database-specific constraints
- Test implementations with both database backends

#### Migration Best Practices
- **Backward Compatibility**: Ensure migrations don't break existing data
- **Rollback Safety**: Always test the `.down.sql` migration
- **Data Preservation**: Use appropriate constraints and indexes
- **Documentation**: Comment complex migrations explaining their purpose

### Configuration Changes

- Update relevant configuration files in `configuration/`
- Update the `Settings` struct in `src/lib/configuration.rs` if needed
- Update environment variable documentation in README.md

## 🔄 Pull Request Process

### Before Submitting a Pull Request

1. **Ensure your fork is up to date**
   ```bash
   git remote add upstream https://github.com/ORIGINAL_OWNER/url-shortener-ztm.git
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Write clean, well-documented code
   - Add tests for new functionality
   - Update documentation as needed

4. **Test your changes**
   ```bash
   # Run all tests
   cargo test

   # Check for linting issues
   cargo clippy

   # Verify code formatting
   cargo fmt --check

   # Test PostgreSQL features (if applicable)
   cargo test postgres_database_insert_get -- --ignored

   # Test rate limiting (if contributing to rate limiting features)
   cargo test rate_limiting
   ```

5. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

### Submitting Your Pull Request

1. **Push your branch**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request** on GitHub with:
   - A clear, descriptive title
   - A detailed description of your changes
   - Reference to any related issues (e.g., "Closes #123")
   - Screenshots or demos for UI changes

3. **Address review feedback** promptly and professionally

### Pull Request Template

When creating a PR, please include:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Other (please describe)

## Testing
- [ ] Tests pass locally
- [ ] New tests added for new functionality
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Code is commented where necessary
- [ ] Documentation updated
- [ ] No merge conflicts
```

## 🗂️ Project Structure

Understanding the codebase structure will help you contribute more effectively:

```
src/
├── bin/main.rs              # Application entry point
└── lib/
    ├── lib.rs               # Library root
    ├── configuration.rs     # Config management
    ├── database/
    │   ├── mod.rs           # Database trait definitions
    │   ├── sqlite.rs        # SQLite implementation
    │   └── postgres_sql.rs  # PostgreSQL implementation
    ├── routes/              # HTTP route handlers
    │   ├── health_check.rs  # Health check endpoint
    │   ├── shorten.rs       # URL shortening (with validation)
    │   ├── redirect.rs      # URL redirection
    │   └── index.rs         # Admin interface
    ├── middleware.rs        # Rate limiting & auth middleware
    ├── startup.rs           # Application startup
    └── ...

tests/api/                   # Integration tests
configuration/               # YAML config files
├── base.yml                # Base configuration
├── local.yml               # Local development
└── production.yaml         # Production settings

migrations/                  # SQLite database migrations
└── pg/                     # PostgreSQL database migrations

templates/                   # HTML templates (Tera)
static/                      # CSS/JS assets
flake.nix                   # Nix development environment
```

## 📋 Issue Labels

- `hacktoberfest` - Issues suitable for Hacktoberfest
- `good first issue` - Great for new contributors
- `help wanted` - We need community help
- `bug` - Something isn't working
- `enhancement` - New feature or request
- `documentation` - Documentation improvements needed
- `database` - Database-related (SQLite/PostgreSQL)
- `rate-limiting` - Rate limiting functionality
- `security` - Security and input validation
- `nix` - Nix development environment
- `testing` - Test improvements or additions
- `performance` - Performance optimizations
- `question` - Further information is requested

## 🐛 Reporting Bugs

When reporting bugs, please include:

- **Environment details** (OS, Rust version, etc.)
- **Steps to reproduce** the issue
- **Expected behavior**
- **Actual behavior**
- **Error messages** or logs
- **Code samples** if applicable

## 💡 Suggesting Features

For feature requests:

- **Check existing issues** first to avoid duplicates
- **Describe the problem** your feature would solve
- **Provide detailed specification** of the proposed solution
- **Consider backwards compatibility**
- **Include mockups or examples** if applicable

## ❓ Getting Help

- **GitHub Issues** - For bugs and feature requests
- **GitHub Discussions** - For questions and general discussion
- **Code comments** - Well-documented codebase for reference

## 📜 Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow GitHub's community guidelines

## 🔧 Special Contribution Areas

### Database Backend Development
The project supports multiple database backends. Areas for contribution:

- **PostgreSQL Enhancements**: Connection pooling, advanced indexing, performance tuning
- **Database Abstraction**: New database implementations (MySQL, Redis, etc.)
- **Migration Tools**: Database switching utilities, data migration scripts
- **Testing**: Cross-database compatibility tests, performance benchmarks

### Rate Limiting and Security
Current rate limiting can be extended:

- **Advanced Algorithms**: Token bucket, sliding window implementations
- **Configuration**: Per-endpoint limits, user-based limits, IP whitelisting
- **Monitoring**: Rate limit metrics, abuse detection, alerting
- **Security**: DDoS protection, bot detection, CAPTCHA integration

### Input Validation and Security
URL validation and security features:

- **URL Validation**: Custom validation rules, domain blocking, content filtering
- **Security Scanning**: Malware detection, phishing protection
- **Input Sanitization**: Enhanced XSS protection, injection prevention
- **Audit Logging**: Security event tracking, compliance features

### Development Environment
Nix and development tooling improvements:

- **CI/CD**: GitHub Actions workflows, automated testing, deployment
- **Development Tools**: IDE integration, debugging tools, profiling
- **Documentation**: Setup guides, troubleshooting, best practices
- **Cross-Platform**: Windows support, Docker integration

## 🎯 Contribution Goals

We're especially interested in contributions that:

- **Performance and Scalability**: Database optimizations, caching, load testing
- **URL Management**: User authentication, custom aliases, analytics, expiration
- **Security and Reliability**: Input validation improvements, rate limiting enhancements
- **Database Features**: PostgreSQL optimizations, connection pooling, migration tools
- **User Experience**: Admin interface improvements, API enhancements
- **Testing**: Multi-database test coverage, performance tests, security tests
- **Development Environment**: Nix flake improvements, CI/CD enhancements
- **Documentation**: API documentation, deployment guides, architecture explanations
- **Rate Limiting**: Customizable limits, IP whitelisting, advanced algorithms
- **Input Validation**: URL parsing improvements, security hardening

## 🏆 Recognition

Contributors will be recognized in:
- GitHub contributors list
- Project documentation
- Release notes for significant contributions

---

Thank you for contributing to the URL Shortener project! Your contributions help make this project better for everyone. 🚀

**Happy Hacking! 🎃**