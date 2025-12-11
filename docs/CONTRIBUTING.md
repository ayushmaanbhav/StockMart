# Contributing to StockMart

First off, thank you for considering contributing to StockMart! It's people like you that make StockMart such a great tool for education and learning.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Pull Request Process](#pull-request-process)
- [Testing Requirements](#testing-requirements)

---

## Code of Conduct

This project and everyone participating in it is governed by our commitment to providing a welcoming and inclusive environment. By participating, you agree to:

- **Be respectful**: Treat everyone with respect and consideration
- **Be constructive**: Provide helpful feedback and accept criticism gracefully
- **Be inclusive**: Welcome newcomers and help them learn
- **Be patient**: Remember that everyone was a beginner once

---

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When creating a bug report, include:

- **Clear title**: Summarize the problem
- **Steps to reproduce**: Detailed steps to reproduce the issue
- **Expected behavior**: What you expected to happen
- **Actual behavior**: What actually happened
- **Environment**: OS, browser, Rust version, Node.js version
- **Screenshots**: If applicable

Use this template:

```markdown
## Bug Description
A clear description of what the bug is.

## Steps to Reproduce
1. Go to '...'
2. Click on '...'
3. See error

## Expected Behavior
What you expected to happen.

## Actual Behavior
What actually happened.

## Environment
- OS: [e.g., macOS 14.0]
- Browser: [e.g., Chrome 120]
- Rust: [e.g., 1.76.0]
- Node.js: [e.g., 20.0.0]

## Screenshots
If applicable, add screenshots.
```

### Suggesting Features

Feature suggestions are welcome! Please include:

- **Use case**: Who would use this feature and why?
- **Proposed solution**: How do you envision it working?
- **Alternatives**: Any alternative solutions you considered?
- **Mockups**: Sketches or screenshots if applicable

### Your First Code Contribution

Not sure where to start? Look for issues labeled:

- `good first issue` - Simple issues for beginners
- `help wanted` - Issues where we need community help
- `documentation` - Improvements to docs

### Pull Requests

We actively welcome your pull requests! Here's how:

1. Fork the repo and create your branch from `main`
2. Make your changes with appropriate tests
3. Ensure the test suite passes
4. Update documentation if needed
5. Submit a pull request

---

## Development Setup

### Prerequisites

| Tool | Version | Installation |
|------|---------|--------------|
| Rust | 1.76+ | [rustup.rs](https://rustup.rs/) |
| Node.js | 18+ | [nodejs.org](https://nodejs.org/) |
| Git | Latest | [git-scm.com](https://git-scm.com/) |

### Clone and Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/stockmart.git
cd stockmart

# Add upstream remote
git remote add upstream https://github.com/original/stockmart.git
```

### Backend Setup

```bash
cd backend

# Install Rust dependencies (automatic with cargo)
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Frontend Setup

```bash
cd frontend

# Install dependencies
npm install

# Run development server
npm run dev

# Run tests
npm test

# Run linter
npm run lint
```

### Running Both

Open two terminals:

```bash
# Terminal 1 - Backend
cd backend && RUST_LOG=info cargo run

# Terminal 2 - Frontend
cd frontend && npm run dev
```

Visit [http://localhost:5174](http://localhost:5174) to see the app.

---

## Code Style

### Rust Style

We follow the standard Rust style guide with `rustfmt`:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy
```

**Guidelines**:

- Use meaningful variable names
- Add doc comments for public items
- Keep functions focused and small
- Use `Result` for fallible operations
- Prefer `?` over explicit error handling when appropriate

```rust
// Good
/// Creates a new order for the specified user.
///
/// # Errors
/// Returns `TradingError::InsufficientFunds` if the user doesn't have enough money.
pub async fn create_order(&self, user_id: UserId, request: OrderRequest) -> Result<Order, TradingError> {
    let user = self.user_repo.find_by_id(user_id)
        .ok_or(TradingError::UserNotFound)?;

    self.validate_order(&user, &request)?;
    self.engine.place_order(request).await
}
```

### TypeScript Style

We use ESLint with the project configuration:

```bash
# Run linter
npm run lint

# Fix auto-fixable issues
npm run lint -- --fix
```

**Guidelines**:

- Use TypeScript strict mode
- Define interfaces for all data structures
- Use functional components with hooks
- Avoid `any` type - use proper typing
- Use `const` by default, `let` when necessary

```typescript
// Good
interface OrderRequest {
    symbol: string;
    side: 'buy' | 'sell' | 'short';
    quantity: number;
    price?: number;
}

const QuickTradeWidget: React.FC<Props> = ({ symbol }) => {
    const [quantity, setQuantity] = useState<number>(10);
    const placeOrder = useGameStore((state) => state.placeOrder);

    const handleSubmit = useCallback(() => {
        placeOrder({ symbol, side: 'buy', quantity });
    }, [symbol, quantity, placeOrder]);

    return (
        // JSX
    );
};
```

### Commit Messages

Use conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting, etc.)
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples**:

```bash
feat(trading): add stop-loss order type
fix(auth): handle session expiry correctly
docs(readme): add architecture diagram
test(engine): add matching edge cases
```

---

## Pull Request Process

### Before Submitting

1. **Update your fork**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks**:
   ```bash
   # Backend
   cd backend
   cargo fmt -- --check
   cargo clippy
   cargo test

   # Frontend
   cd frontend
   npm run lint
   npm test
   ```

3. **Update documentation** if you changed:
   - API endpoints or messages
   - Configuration options
   - User-facing features

### PR Template

```markdown
## Description
Brief description of changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Added unit tests
- [ ] Added integration tests
- [ ] All existing tests pass

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-reviewed my code
- [ ] Commented hard-to-understand areas
- [ ] Updated documentation
- [ ] No new warnings
```

### Review Process

1. A maintainer will review your PR
2. Address any requested changes
3. Once approved, a maintainer will merge
4. Delete your branch after merge

---

## Testing Requirements

### Backend Tests

All new backend code should have tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_place_order_success() {
        let state = create_test_state().await;
        let order = OrderRequest {
            symbol: "AAPL".to_string(),
            side: OrderSide::Buy,
            quantity: 10,
            price: Some(1500000),
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::GTC,
        };

        let result = state.engine.place_order(order).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_place_order_insufficient_funds() {
        let state = create_test_state().await;
        // Set user money to 0
        // Try to place order
        // Assert InsufficientFunds error
    }
}
```

### Frontend E2E Tests

For user-facing features, add Playwright tests:

```typescript
// tests/e2e/trading/new-feature.spec.ts
import { test, expect } from '@playwright/test';

test.describe('New Feature', () => {
    test('should work correctly', async ({ page }) => {
        // Arrange
        await page.goto('/trade');
        await loginAsTrader(page);

        // Act
        await page.click('[data-testid="new-feature-button"]');

        // Assert
        await expect(page.locator('.result')).toBeVisible();
    });
});
```

### Test Coverage Goals

| Area | Target |
|------|--------|
| Domain logic | 90%+ |
| Service layer | 80%+ |
| API handlers | 70%+ |
| UI components | Key flows covered |

---

## Questions?

Feel free to open an issue with the `question` label if you need help!

---

Thank you for contributing to StockMart! Your efforts help make trading education accessible to everyone.
