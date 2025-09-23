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

### Setting Up Your Development Environment

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

## 🤝 How to Contribute

### Types of Contributions Welcome

- 🐛 **Bug fixes** - Fix issues in the codebase
- ✨ **New features** - Add functionality to the URL shortener
- 📚 **Documentation** - Improve README, code comments, or create tutorials
- 🧪 **Tests** - Add or improve test coverage
- 🎨 **UI/UX improvements** - Enhance the web interface
- ⚡ **Performance optimizations** - Make the service faster or more efficient
- 🔒 **Security enhancements** - Improve the security of the application
- 🏗️ **Refactoring** - Improve code structure and maintainability

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
# Run all tests
cargo test

# Run tests with logging output (useful for debugging)
TEST_LOG=1 cargo test

# Run specific test modules
cargo test health_check
cargo test shorten
cargo test redirect
```

### Database Changes

If your contribution involves database changes:

1. Create a new migration file in the `migrations/` directory
2. Follow the existing naming convention: `YYYYMMDDHHMMSS_description.up.sql` and `.down.sql`
3. Test your migration with `sqlx migrate run` and `sqlx migrate revert`

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
   cargo test
   cargo clippy
   cargo fmt --check
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
    ├── database/            # Database layer
    ├── routes/              # HTTP route handlers
    ├── startup.rs           # Application startup
    └── ...

tests/api/                   # Integration tests
configuration/               # YAML config files
migrations/                  # Database migrations
templates/                   # HTML templates
static/                      # CSS/JS assets
```

## 📋 Issue Labels

- `hacktoberfest` - Issues suitable for Hacktoberfest
- `good first issue` - Great for new contributors
- `help wanted` - We need community help
- `bug` - Something isn't working
- `enhancement` - New feature or request
- `documentation` - Documentation improvements needed
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

## 🎯 Contribution Goals

We're especially interested in contributions that:

- Improve performance and scalability
- Add useful features for URL management
- Enhance security and reliability
- Improve user experience
- Add comprehensive tests
- Improve documentation and examples

## 🏆 Recognition

Contributors will be recognized in:
- GitHub contributors list
- Project documentation
- Release notes for significant contributions

---

Thank you for contributing to the URL Shortener project! Your contributions help make this project better for everyone. 🚀

**Happy Hacking! 🎃**