# Contributing to Ceres Network

Thank you for your interest in contributing to Ceres Network! This document provides guidelines and instructions for contributing to the project.

## 🌟 Ways to Contribute

- **Code**: Fix bugs, add features, improve performance
- **Documentation**: Improve guides, add examples, fix typos
- **Testing**: Write tests, report bugs, test on different platforms
- **Design**: Improve UX, create diagrams, design interfaces
- **Community**: Answer questions, help onboard new contributors

## 🚀 Getting Started

### Prerequisites

- Rust 1.81.0 (automatically installed via `rust-toolchain.toml`)
- Stellar CLI for deployment
- Node.js 20+ for TypeScript SDK development
- Git for version control

### Setup Development Environment

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/ceres-network.git
cd ceres-network

# Add upstream remote
git remote add upstream https://github.com/ceres-network/ceres-network.git

# Install Rust dependencies
cargo build

# Install TypeScript dependencies
cd sdk/typescript
npm install
cd ../..
```

## 🧪 Running Tests Locally

### Rust Contract Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test pool_tests

# Run with output
cargo test -- --nocapture

# Run with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### TypeScript SDK Tests

```bash
cd sdk/typescript

# Run tests
npm test

# Type check
npm run typecheck

# Lint
npm run lint
```

### Format and Lint

```bash
# Format Rust code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Fix clippy warnings automatically
cargo clippy --fix
```

## 📝 Pull Request Process

### Before Submitting

1. **Create an issue** first to discuss major changes
2. **Fork the repository** and create a feature branch
3. **Write tests** for new functionality
4. **Update documentation** if you change APIs
5. **Run all tests** and ensure they pass
6. **Format your code** with `cargo fmt`
7. **Check for warnings** with `cargo clippy`

### PR Checklist

- [ ] Tests pass locally (`cargo test`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Documentation is updated
- [ ] Commit messages are clear and descriptive
- [ ] PR description explains what and why
- [ ] Related issue is linked (if applicable)

### PR Title Format

Use conventional commit format:

- `feat: add flood trigger support`
- `fix: correct collateral ratio calculation`
- `docs: update oracle specification`
- `test: add integration test for partial payouts`
- `refactor: simplify policy state management`
- `chore: update dependencies`

### PR Description Template

```markdown
## Description
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- List of specific changes
- Another change

## Testing
How was this tested?

## Related Issues
Closes #123
```

## 🏷️ Issue Labels

- **good first issue**: Great for newcomers
- **help wanted**: Extra attention needed
- **bug**: Something isn't working
- **enhancement**: New feature or request
- **documentation**: Documentation improvements
- **oracle**: Oracle-related changes
- **contracts**: Smart contract changes
- **sdk**: TypeScript SDK changes
- **security**: Security-related issues

## 🎯 Development Guidelines

### Rust Smart Contracts

1. **No `unwrap()` in production code**
   - Use `?` operator with typed errors
   - Handle all error cases explicitly

2. **Use typed errors**
   ```rust
   #[contracttype]
   pub enum Error {
       NotFound = 1,
       Unauthorized = 2,
   }
   ```

3. **Storage keys must use `Symbol`**
   ```rust
   #[contracttype]
   pub enum DataKey {
       Config,
       Policy(u64),
   }
   ```

4. **Add doc comments to public functions**
   ```rust
   /// Register a new parametric insurance policy
   pub fn register_policy(env: Env, ...) -> Result<u64, Error> {
   ```

5. **Write unit tests for all functions**
   ```rust
   #[test]
   fn test_policy_registration() {
       // Test implementation
   }
   ```

### TypeScript SDK

1. **Use TypeScript strict mode**
2. **Export all public types**
3. **Add JSDoc comments**
   ```typescript
   /**
    * Register a new parametric insurance policy
    * @param keypair - Farmer's keypair
    * @param params - Policy parameters
    * @returns Policy ID
    */
   async registerPolicy(keypair: Keypair, params: RegisterPolicyParams): Promise<bigint>
   ```

4. **Handle errors gracefully**
5. **Write examples in documentation**

### Testing Standards

1. **Test happy paths**
2. **Test error conditions**
3. **Test edge cases**
4. **Use descriptive test names**
   ```rust
   #[test]
   fn test_pool_rejects_policy_when_collateral_ratio_breached() {
   ```

5. **Keep tests isolated and independent**

## 🔒 Security Guidelines

1. **Never commit secrets or private keys**
2. **Validate all inputs**
3. **Check authorization before state changes**
4. **Use safe math operations**
5. **Report security issues privately** to security@ceres.network

## 📚 Documentation Standards

1. **Keep README.md up to date**
2. **Document all public APIs**
3. **Include code examples**
4. **Explain complex logic with comments**
5. **Update architecture docs when changing design**

## 🤝 Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of:

- Age, body size, disability, ethnicity
- Gender identity and expression
- Level of experience
- Nationality, personal appearance, race
- Religion, sexual identity and orientation

### Our Standards

**Positive behavior includes:**

- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable behavior includes:**

- Trolling, insulting/derogatory comments, personal attacks
- Public or private harassment
- Publishing others' private information without permission
- Other conduct which could reasonably be considered inappropriate

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported to the project team at conduct@ceres.network. All complaints will be reviewed and investigated promptly and fairly.

## 💬 Communication

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, general discussion
- **Discord**: Real-time chat (link in README)
- **Twitter**: [@CeresNetwork](https://twitter.com/CeresNetwork)

## 🎓 Learning Resources

### Stellar & Soroban

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Developer Docs](https://developers.stellar.org)
- [Soroban Examples](https://github.com/stellar/soroban-examples)

### Rust

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

### Parametric Insurance

- [Parametric Insurance Explained](https://www.investopedia.com/terms/p/parametric-insurance.asp)
- [Index-Based Agricultural Insurance](https://www.worldbank.org/en/topic/financialsector/brief/index-based-crop-insurance)

## 🙏 Recognition

Contributors will be:

- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Eligible for contributor NFTs (coming soon)

## ❓ Questions?

Don't hesitate to ask! Open an issue with the `question` label or reach out on Discord.

---

Thank you for contributing to Ceres Network! 🌾
