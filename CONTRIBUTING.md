# Contributing to DuxNet

Thank you for your interest in contributing to DuxNet! This document provides guidelines and information for contributors.

## 🤝 How to Contribute

### Reporting Issues
- Use the GitHub issue tracker
- Provide detailed information about the problem
- Include steps to reproduce the issue
- Mention your operating system and Rust version

### Suggesting Features
- Open a feature request issue
- Describe the feature and its benefits
- Consider implementation complexity
- Discuss with the community first

### Code Contributions
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Ensure code quality
6. Submit a pull request

## 🛠️ Development Setup

### Prerequisites
- Rust 1.70+ (latest stable)
- Node.js 16+ (for Tauri)
- Git

### Local Development
```bash
# Clone your fork
git clone https://github.com/your-username/duxnet-platform.git
cd duxnet-platform

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run
```

### Code Style
- Follow Rust formatting guidelines
- Use `cargo fmt` to format code
- Use `cargo clippy` for linting
- Write meaningful commit messages

## 📝 Code Guidelines

### Rust Code
- Use meaningful variable and function names
- Add comments for complex logic
- Handle errors appropriately
- Write unit tests for new functionality
- Follow Rust idioms and best practices

### API Design
- Use RESTful principles
- Provide clear error messages
- Include proper HTTP status codes
- Document new endpoints

### Security
- Never commit sensitive data
- Validate all inputs
- Use secure cryptographic practices
- Follow the principle of least privilege

## 🧪 Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Test Guidelines
- Write tests for new functionality
- Ensure good test coverage
- Use descriptive test names
- Test both success and failure cases

## 📚 Documentation

### Code Documentation
- Document public APIs
- Use Rust doc comments
- Provide examples where helpful
- Keep documentation up-to-date

### User Documentation
- Update README.md for user-facing changes
- Add installation instructions for new dependencies
- Document configuration options
- Provide troubleshooting guides

## 🔄 Pull Request Process

1. **Create a branch**: Use descriptive branch names
2. **Make changes**: Follow coding guidelines
3. **Test thoroughly**: Ensure all tests pass
4. **Update documentation**: Include relevant docs
5. **Submit PR**: Provide clear description

### PR Guidelines
- Use descriptive titles
- Explain the changes clearly
- Reference related issues
- Include screenshots for UI changes
- Ensure CI checks pass

## 🏷️ Issue Labels

- `bug`: Something isn't working
- `enhancement`: New feature or request
- `documentation`: Improvements to documentation
- `good first issue`: Good for newcomers
- `help wanted`: Extra attention needed
- `priority: high`: High priority issues

## 🎯 Areas for Contribution

### High Priority
- Bug fixes and stability improvements
- Performance optimizations
- Security enhancements
- Documentation improvements

### Medium Priority
- New features and functionality
- UI/UX improvements
- Testing improvements
- Code refactoring

### Low Priority
- Nice-to-have features
- Cosmetic changes
- Experimental features

## 📞 Getting Help

- **Discussions**: Use GitHub Discussions
- **Issues**: Open an issue for bugs
- **Documentation**: Check the docs folder
- **Community**: Join our community channels

## 🏆 Recognition

Contributors will be recognized in:
- Repository contributors list
- Release notes
- Project documentation
- Community acknowledgments

## 📄 License

By contributing to DuxNet, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to DuxNet! Your help makes this project better for everyone. 