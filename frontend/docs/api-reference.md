# API Reference

HackFlare provides a REST API for automating DNS management.

## Base URL

All API requests should be made to:

```
https://hackflare.app/api/v1
```

## Authentication

Include your API key in the `Authorization` header:

```
Authorization: Bearer <your-api-key>
```

You can generate API keys from the [Settings](/dash/settings) page.

## Endpoints

### Health Check

```
GET /health
```

Returns the service status.

### Zones

```
GET    /dns/zones              List all zones
POST   /dns/zones              Create a new zone
GET    /dns/zones/:id          Get zone details
DELETE /dns/zones/:id          Delete a zone
```

### DNS Records

```
GET    /dns/zones/:id/records     List records
POST   /dns/zones/:id/records     Create a record
PUT    /dns/zones/:id/records/:record  Update a record
DELETE /dns/zones/:id/records/:record  Delete a record
```

### Users

```
GET    /users/me       Get current user
PUT    /users/me       Update current user
```

## Rate Limiting

API requests are rate-limited to 100 requests per minute per API key.

## Errors

The API uses conventional HTTP response codes:

| Code | Meaning |
|------|---------|
| 200 | Success |
| 400 | Bad request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not found |
| 429 | Rate limit exceeded |
| 500 | Internal server error |
