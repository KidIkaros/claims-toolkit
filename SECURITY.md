# Security Guidelines for claims-toolkit

## Input Validation

### File Path Security
- **Directory Traversal Protection**: Rejects paths containing `..` sequences
- **System Directory Protection**: Blocks access to sensitive system directories (`/etc/`, `/var/`, `/usr/`, `/sys/`, `/proc/`, `/dev/`, `/root/`)
- **Path Sanitization**: All file paths are validated before access

### File Size Limits
- **Maximum File Size**: 10 MB limit for input files to prevent DoS attacks
- **Content Validation**: Basic size checks on content to prevent memory exhaustion
- **Buffer Limits**: Stdin input also subject to size validation

## Error Handling

### Sanitized Error Messages
- **No Path Leakage**: Error messages don't expose internal file system paths
- **Truncation**: Long error messages are truncated to prevent information disclosure
- **Generic Messages**: File access errors return generic messages like "Cannot access file" instead of detailed OS errors

## API Security (When Enabled)

### CORS Configuration
- **CORS Enabled**: Cross-Origin Resource Sharing enabled for API endpoints
- **Flexible Origins**: Allows requests from any origin (customize for production)

### Request Validation
- **JSON Schema**: API requests validated against expected schemas
- **Size Limits**: API request bodies subject to same size limits as file uploads

## Best Practices

### Secrets Management
- **No Hardcoded Secrets**: No API keys or credentials in source code
- **Environment Variables**: Use environment variables for configuration
- **Credential Rotation**: Implement regular credential rotation in production

### Data Protection
- **PHI Handling**: PHI scanner uses secure in-memory processing
- **No Logging of PHI**: Ensure logs don't contain Protected Health Information
- **Secure Defaults**: Conservative defaults for all security-related settings

## Audit Recommendations

1. **Regular Dependency Updates**: Run `cargo audit` to check for vulnerabilities
2. **File Permissions**: Ensure output directories have appropriate permissions
3. **Network Security**: If running API mode, use reverse proxy with TLS
4. **Monitoring**: Log security events (failed access attempts, size violations)

## Security Testing

Run security-focused tests:
```bash
cargo test --release security
cargo audit
```

## Reporting Security Issues

Please report security vulnerabilities to the project maintainers. Do not open public issues for security bugs.
