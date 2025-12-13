# Security Policy

## Supported Versions

We actively support the following versions of StockMart with security updates:

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of StockMart seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

**Please DO NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them through one of these channels:

1. **GitHub Security Advisories** (Preferred)
   - Go to the [Security Advisories](../../security/advisories/new) page
   - Click "Report a vulnerability"
   - Fill out the form with details

2. **Private Disclosure**
   - If you cannot use GitHub Security Advisories, contact the maintainers directly
   - Include `[SECURITY]` in your communication subject

### What to Include

Please include the following information in your report:

- **Type of vulnerability** (e.g., XSS, SQL injection, authentication bypass)
- **Location** - Full path to the affected source file(s)
- **Configuration** - Any special configuration required to reproduce
- **Steps to reproduce** - Detailed steps to reproduce the vulnerability
- **Proof of concept** - If possible, include code or screenshots
- **Impact assessment** - What an attacker could achieve by exploiting this
- **Suggested fix** - If you have recommendations for fixing the issue

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution Target**: Within 30 days for critical issues

### What to Expect

1. **Acknowledgment**: We will acknowledge your report within 48 hours
2. **Investigation**: We will investigate and keep you informed of our progress
3. **Fix Development**: We will work on a fix and may ask for your input
4. **Disclosure**: We will coordinate disclosure timing with you
5. **Credit**: We will credit you in our security advisories (unless you prefer anonymity)

## Security Best Practices for Deployers

When deploying StockMart, please follow these security guidelines:

### Authentication & Authorization
- Always use strong passwords for admin accounts
- Enable rate limiting for login attempts
- Use HTTPS in production environments

### Environment Configuration
- Never commit `.env` files or credentials to version control
- Use environment variables for sensitive configuration
- Rotate secrets and API keys regularly

### Network Security
- Deploy behind a reverse proxy (nginx, Caddy, etc.)
- Configure proper CORS settings for your domain
- Use firewall rules to restrict access to backend ports

### Database Security
- Use parameterized queries (already implemented in our Rust backend)
- Regular backups of game data
- Restrict database access to application servers only

### WebSocket Security
- Validate all incoming WebSocket messages
- Implement connection limits per IP
- Use WSS (WebSocket Secure) in production

## Security Features

StockMart includes the following security features:

- **Input Validation**: All user inputs are validated on both client and server
- **SQL Injection Prevention**: Parameterized queries via SQLx
- **XSS Protection**: React's built-in XSS protection + Content Security Policy
- **Rate Limiting**: Configurable rate limits for API endpoints
- **Session Management**: Secure session handling with proper timeouts
- **CORS Configuration**: Strict CORS policies for API access

## Scope

The following are in scope for security reports:

- StockMart backend (Rust)
- StockMart frontend (React/TypeScript)
- Trading engine and order matching
- Authentication and session management
- WebSocket communication
- Admin dashboard

The following are out of scope:

- Third-party dependencies (report to upstream maintainers)
- Issues in deployment infrastructure you manage
- Social engineering attacks
- Physical security

## Recognition

We maintain a list of security researchers who have responsibly disclosed vulnerabilities:

*No vulnerabilities reported yet - be the first!*

---

Thank you for helping keep StockMart and its users safe!
