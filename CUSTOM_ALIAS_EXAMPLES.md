# Custom Alias Examples

This document demonstrates how to use the new custom alias functionality in the URL shortener service.

## API Endpoints

The URL shortener now supports custom aliases through the following endpoints:
- `POST /api/shorten` (protected - requires API key)
- `POST /api/public/shorten` (public - no authentication required)

## Request Format

The request body should be JSON with the following structure:

```json
{
  "url": "https://www.example.com/very/long/url",
  "alias": "my-custom-link"  // Optional
}
```

## Examples

### 1. Shorten URL with Custom Alias

```bash
curl -X POST http://localhost:8000/api/shorten \
  -H "x-api-key: your-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.example.com/very/long/url",
    "alias": "my-custom-link"
  }'
```

**Response:**
```json
{
  "success": true,
  "message": "ok",
  "status": 200,
  "time": "2025-01-18T12:00:00Z",
  "data": {
    "shortened_url": "https://localhost/my-custom-link",
    "original_url": "https://www.example.com/very/long/url",
    "id": "my-custom-link"
  }
}
```

### 2. Shorten URL without Custom Alias (Auto-generated)

```bash
curl -X POST http://localhost:8000/api/shorten \
  -H "x-api-key: your-api-key-here" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.example.com/very/long/url"
  }'
```

**Response:**
```json
{
  "success": true,
  "message": "ok",
  "status": 200,
  "time": "2025-01-18T12:00:00Z",
  "data": {
    "shortened_url": "https://localhost/AbC123",
    "original_url": "https://www.example.com/very/long/url",
    "id": "AbC123"
  }
}
```

### 3. Public Endpoint (No API Key Required)

```bash
curl -X POST http://localhost:8000/api/public/shorten \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.example.com/very/long/url",
    "alias": "public-link"
  }'
```

## Custom Alias Validation Rules

Custom aliases must meet the following requirements:

### Valid Characters
- Letters (A-Z, a-z)
- Numbers (0-9)
- Underscores (_)
- Hyphens (-)

### Length Requirements
- Minimum: 1 character
- Maximum: 50 characters

### Invalid Examples
- `admin` (reserved word)
- `invalid@alias` (invalid character @)
- `invalid alias` (space character)
- `_invalid` (starts with underscore)
- `invalid_` (ends with underscore)
- `-invalid` (starts with hyphen)
- `invalid-` (ends with hyphen)
- `test__link` (consecutive underscores)
- `test--link` (consecutive hyphens)
- `a`.repeat(51) (too long)

### Valid Examples
- `my-link`
- `project_2024`
- `ABC123`
- `test-123`
- `valid_alias`
- `a` (minimum length)
- `a`.repeat(50) (maximum length)

## Error Responses

### Alias Already in Use
```json
{
  "success": false,
  "message": "Alias 'my-custom-link' is already in use",
  "status": 422,
  "time": "2025-01-18T12:00:00Z",
  "data": null
}
```

### Invalid Alias Format
```json
{
  "success": false,
  "message": "Alias can only contain letters (A-Z, a-z), numbers (0-9), underscores (_), and hyphens (-)",
  "status": 422,
  "time": "2025-01-18T12:00:00Z",
  "data": null
}
```

### Reserved Alias
```json
{
  "success": false,
  "message": "Alias 'admin' is reserved and cannot be used",
  "status": 422,
  "time": "2025-01-18T12:00:00Z",
  "data": null
}
```

## Reserved Aliases

The following aliases are reserved and cannot be used:
- `admin`
- `api`
- `static`
- `health`
- `health_check`
- `login`
- `register`
- `dashboard`
- `profile`
- `logout`
- `shorten`
- `redirect`
- `users`
- `tags`
- `public`
- `help`
- `about`
- `contact`
- `terms`
- `privacy`
- `favicon.ico`
- `robots.txt`
- `sitemap.xml`

## Benefits

1. **User-Friendly URLs**: Create memorable, branded short URLs
2. **Flexibility**: Choose between custom aliases or auto-generated IDs
3. **Validation**: Comprehensive validation ensures URL safety and usability
4. **Uniqueness**: Automatic checking prevents duplicate aliases
5. **Backwards Compatibility**: Existing functionality remains unchanged

## Migration Notes

- The API now expects JSON requests instead of plain text
- Custom aliases are optional - existing behavior is preserved when no alias is provided
- All validation is performed server-side for security
- Database schema remains unchanged (still uses the `id` field for both auto-generated and custom aliases)
