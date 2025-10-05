# Contributing to Jupiter DEX Events Substream

Thank you for your interest in contributing to the Jupiter DEX Events Substream! This document provides guidelines for contributing to the project.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+
- Substreams CLI
- Git
- Solana CLI (optional)

### Development Setup

1. **Fork the repository**
   ```bash
   git clone https://github.com/PaulieB14/Jupiter-Dex-Substreams.git
   cd Jupiter-Dex-Substreams
   ```

2. **Install dependencies**
   ```bash
   # Install Substreams CLI
   curl -sSL https://substreams.dev/install.sh | bash
   
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project**
   ```bash
   substreams build
   ```

## 🛠️ Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 3. Test Your Changes

```bash
# Run the substream locally
substreams run substreams.yaml jupiter_events \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +1

# Run with GUI for debugging
substreams gui substreams.yaml jupiter_events
```

### 4. Commit Your Changes

```bash
git add .
git commit -m "feat: add your feature description"
```

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## 📝 Code Style Guidelines

### Rust Code

- Use `cargo fmt` to format code
- Use `cargo clippy` for linting
- Follow Rust naming conventions
- Add documentation comments for public functions

### Protobuf Schema

- Use descriptive field names
- Add comments for complex fields
- Follow protobuf naming conventions
- Maintain backward compatibility

### Documentation

- Update README.md for user-facing changes
- Update docs/ for architectural changes
- Add inline comments for complex logic
- Keep documentation up to date

## 🧪 Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
# Test with real blockchain data
substreams run substreams.yaml jupiter_events \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +10
```

### Performance Tests

```bash
# Test with larger block ranges
substreams run substreams.yaml jupiter_events \
  -e mainnet.sol.streamingfast.io:443 \
  -s 325766951 -t +100
```

## 📋 Pull Request Guidelines

### Before Submitting

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] Documentation is updated
- [ ] Changes are well-documented
- [ ] No breaking changes (or clearly documented)

### PR Description

Include:
- **What**: Description of changes
- **Why**: Motivation for changes
- **How**: Implementation details
- **Testing**: How changes were tested

### Example PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes
```

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Environment**: OS, Rust version, Substreams version
2. **Steps to Reproduce**: Clear reproduction steps
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happens
5. **Logs**: Relevant error logs or output
6. **Screenshots**: If applicable

## 💡 Feature Requests

When requesting features, please include:

1. **Use Case**: Why is this feature needed?
2. **Proposed Solution**: How should it work?
3. **Alternatives**: Other approaches considered
4. **Additional Context**: Any other relevant information

## 📚 Documentation

### Code Documentation

- Add doc comments for public functions
- Use `///` for Rust documentation
- Include examples in documentation
- Keep documentation up to date

### README Updates

- Update installation instructions
- Add new usage examples
- Update feature lists
- Keep links current

### Architecture Documentation

- Update `docs/ARCHITECTURE.md` for structural changes
- Document new components
- Update diagrams if applicable
- Keep architecture current

## 🔄 Release Process

### Version Bumping

- **Patch** (0.1.1 → 0.1.2): Bug fixes
- **Minor** (0.1.0 → 0.2.0): New features
- **Major** (0.1.0 → 1.0.0): Breaking changes

### Release Checklist

- [ ] All tests pass
- [ ] Documentation updated
- [ ] Version bumped
- [ ] Changelog updated
- [ ] Package published to registry

## 🤝 Community Guidelines

### Be Respectful

- Use welcoming and inclusive language
- Be respectful of differing viewpoints
- Focus on what is best for the community
- Show empathy towards other community members

### Be Constructive

- Provide constructive feedback
- Help others learn and grow
- Share knowledge and experience
- Be patient with newcomers

## 📞 Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and discussions
- **Substreams Community**: For general Substreams questions
- **Jupiter Community**: For Jupiter-specific questions

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to the Jupiter DEX Events Substream! 🚀
