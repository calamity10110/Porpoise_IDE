# Contributing to Porpoise

## Quick Start

```bash
# 1. Clone
git clone https://github.com/porpoise-ide/porpoise.git
cd porpoise

# 2. Setup
just setup

# 3. Build
just build

# 4. Test
just test
```

## Development Workflow

1. Create a branch: `git checkout -b feature/my-feature`
2. Make changes
3. Run checks: `just check`
4. Commit: `git commit -m "feat: my feature"`
5. Push and create PR

## Code Style

- Run `just fmt-fix` before committing
- All warnings are errors (`-D warnings`)
- Use `#[must_use]` on return values
- Document public APIs with `///` comments

## Testing

```bash
# Run all tests
just test

# Run specific crate tests
just test-p porpoise-core

# Generate coverage report
just test-coverage
```

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation
- `refactor:` code refactor
- `test:` add tests
- `chore:` maintenance

## Pull Requests

- Keep PRs small and focused
- Include tests for new features
- Update documentation if needed
- Ensure CI passes before requesting review
