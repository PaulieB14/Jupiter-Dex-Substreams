# JWT Testing for Jupiter DEX Substreams

This document explains how to test your Jupiter DEX Substreams with JWT authentication in a secure manner.

## 🔒 Security First

**NEVER commit JWT tokens to the repository!** Always use environment variables for sensitive data.

## Getting Started

### 1. Get a JWT Token

1. Visit [substreams.dev](https://substreams.dev)
2. Login to your account
3. Generate an API token
4. Copy the token (it will look like: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`)

### 2. Set Environment Variables

```bash
# Set your JWT token
export SUBSTREAMS_API_TOKEN="your-jwt-token-here"

# Or use the alternative variable
export JWT_TOKEN="your-jwt-token-here"
```

### 3. Run JWT Tests

```bash
# Make the test script executable
chmod +x test/test_jwt.sh

# Run the JWT tests
./test/test_jwt.sh
```

## Test Coverage

The JWT testing suite includes:

### 1. Authentication Tests
- **JWT Token Validation**: Verifies token format and structure
- **Authentication Flow**: Tests the complete auth process
- **Error Handling**: Tests invalid token scenarios

### 2. Foundational Store Tests
- **Account Owner Resolution**: Tests SPL account ownership lookup
- **Trading Data Access**: Tests Jupiter trading data retrieval
- **Token Price Queries**: Tests token price data access
- **Performance Metrics**: Measures response times and throughput

### 3. Integration Tests
- **Jupiter Instructions**: Tests instruction processing with authentication
- **Jupiter Analytics**: Tests analytics generation with JWT
- **End-to-End Flow**: Tests complete data flow from ingestion to analytics

## Test Results

The tests will output:

- ✅ **Authenticated Requests**: Number of successful authentications
- ❌ **Failed Authentications**: Number of failed auth attempts
- 🏪 **Foundational Store Access**: Number of store queries
- 📊 **Performance Metrics**: Response times and throughput data

## Environment Variables

Create a `.env` file (never commit this):

```bash
# Copy the example file
cp .env.example .env

# Edit with your values
nano .env
```

## Security Best Practices

1. **Never commit JWT tokens** to the repository
2. **Use environment variables** for all sensitive data
3. **Rotate tokens regularly** for security
4. **Use different tokens** for different environments
5. **Monitor token usage** for suspicious activity

## Troubleshooting

### Common Issues

1. **"No JWT token found"**
   - Ensure `SUBSTREAMS_API_TOKEN` is set
   - Check that the token is valid and not expired

2. **"Authentication failed"**
   - Verify the token is correct
   - Check that the token has the right permissions

3. **"Invalid token signature"**
   - Ensure the token is properly formatted
   - Check that the token hasn't been tampered with

### Debug Mode

Run with debug logging:

```bash
RUST_LOG=debug ./test/test_jwt.sh
```

## Performance Testing

The JWT tests include performance metrics:

- **Response Time**: Time to process each request
- **Throughput**: Requests per second
- **Memory Usage**: Memory consumption during tests
- **Error Rate**: Percentage of failed requests

## Continuous Integration

For CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run JWT Tests
  env:
    SUBSTREAMS_API_TOKEN: ${{ secrets.SUBSTREAMS_API_TOKEN }}
  run: ./test/test_jwt.sh
```

## Next Steps

After successful JWT testing:

1. **Deploy to substreams.dev** with your authenticated setup
2. **Monitor performance** in production
3. **Set up alerts** for authentication failures
4. **Document your JWT setup** for your team

## Support

If you encounter issues:

1. Check the [Substreams Documentation](https://docs.streamingfast.io/substreams)
2. Review the [JWT Testing Guide](FOUNDATIONAL_STORES.md)
3. Join the [Substreams Discord](https://discord.gg/streamingfast)
4. Open an issue in this repository