# Step 3: OAuth2 + Real Mimir Setup

> ⚠️ **DRAFT — endpoints not yet verified against real Yggdrasil/Mimir.**
> The exact OAuth2 registration API, token endpoint paths, and ConfigMap
> structure below are best-guess based on common OIDC patterns. Before
> running these commands against production infra (June 2-11), confirm
> the actual schema with the Asgard infra team or by reading Yggdrasil's
> own README/source. Treat this file as a checklist scaffold, not a script.

**Status:** Draft scaffold for June 2-11 execution  
**Confidence:** 5/10 (requires infrastructure verification)
**Fallback:** Mock server (`scripts/mock-mimir-server.py`) — already verified

---

## Quick Start (< 15 minutes)

If you have infrastructure access:

```bash
# 1. Register OAuth2 client in Yggdrasil
curl -X POST http://yggdrasil.asgard.internal/register \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "refgraph-s1",
    "client_secret": "YOUR_SECRET_HERE",
    "redirect_uris": ["http://localhost:8000/callback"],
    "grant_types": ["client_credentials"]
  }'

# 2. Export credentials to environment
export MIMIR_TOKEN="<oauth2_access_token>"
export MIMIR_URL="http://mimir.asgard.internal:5000"

# 3. Test connectivity
./scripts/may-18-integration-test.sh --check-mimir-only

# 4. Run data preparation
./scripts/may-23-28-execute.sh
```

---

## Full Setup Guide

### Prerequisites

Before setting up OAuth2 and Real Mimir:

- [ ] Access to Yggdrasil admin console (http://yggdrasil.asgard.internal)
- [ ] Asgard tenant `asgard_insurance` exists in Mimir (verified via `/health` endpoint)
- [ ] Mimir service running on internal network or port-forwarded locally
- [ ] Network connectivity to Mimir (firewall rules configured)
- [ ] OAuth2 credentials available or ability to generate new ones

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ RefGraph CLI (orchestrate binary)                              │
│ - Reads MIMIR_TOKEN from environment                           │
│ - Reads MIMIR_URL (default: http://localhost:5000)             │
└──────────────────────┬──────────────────────────────────────────┘
                       │
                       ↓ (Bearer token via Authorization header)
┌─────────────────────────────────────────────────────────────────┐
│ Mimir API Gateway (port 5000 by default)                        │
│ - Validates JWT token from Yggdrasil                           │
│ - Routes to ingest/search endpoints                            │
│ - Returns 401 if token missing or invalid                      │
└──────────────────────┬──────────────────────────────────────────┘
                       │
                       ↓ (Validates JWT signature)
┌─────────────────────────────────────────────────────────────────┐
│ Yggdrasil OIDC Provider (port 9999 or 8443)                    │
│ - Stores OAuth2 client registrations                           │
│ - Issues JWT access tokens                                    │
│ - Client credentials: refgraph-s1 + secret                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Method 1: Via Yggdrasil Admin Console (Recommended)

### Steps

1. **Access Yggdrasil Admin**
   ```bash
   # If using Tailscale/VPN
   open http://yggdrasil.asgard.internal/admin
   
   # If port-forwarded to localhost
   open http://localhost:9999/admin
   ```

2. **Create New OAuth2 Client**
   - Click "Clients" → "New Client"
   - Fill in:
     - **Client ID:** `refgraph-s1`
     - **Client Secret:** (auto-generated, save it)
     - **Grant Types:** Select `client_credentials`
     - **Redirect URIs:** (not needed for client_credentials)
     - **Scope:** `mimir:read` and `mimir:write`
   - Click "Save"

3. **Get Access Token**
   ```bash
   curl -X POST http://yggdrasil.asgard.internal/oauth/token \
     -H "Content-Type: application/x-www-form-urlencoded" \
     -d "grant_type=client_credentials&client_id=refgraph-s1&client_secret=YOUR_SECRET"
   
   # Response:
   {
     "access_token": "eyJhbGciOiJIUzI1NiIs...",
     "token_type": "Bearer",
     "expires_in": 3600
   }
   ```

4. **Test with Mimir**
   ```bash
   export MIMIR_TOKEN="<access_token_from_step_3>"
   
   curl -X GET http://mimir.asgard.internal:5000/api/v1/tenants/asgard_insurance/health \
     -H "Authorization: Bearer $MIMIR_TOKEN"
   
   # Expected: {"status": "healthy", "entities": 0, "relationships": 0}
   ```

---

## Method 2: Via Kubernetes Secret (For Local OrbStack)

If running Asgard in OrbStack K8s with Yggdrasil as a K8s pod:

### Prerequisites

```bash
# Verify kubectl access to Asgard K8s
kubectl cluster-info
kubectl get pods -n asgard | grep yggdrasil

# Should show: yggdrasil-0 pod running
```

### Steps

1. **Generate OAuth2 Client Secret**
   ```bash
   # Create random secret
   CLIENT_SECRET=$(openssl rand -hex 16)
   echo "Client Secret: $CLIENT_SECRET"
   ```

2. **Update Yggdrasil ConfigMap**
   ```bash
   # Get current config
   kubectl get cm -n asgard yggdrasil-config -o yaml > ygg-config.yaml
   
   # Edit ygg-config.yaml and add to oauth2.clients:
   # - client_id: refgraph-s1
   #   client_secret: <CLIENT_SECRET_FROM_STEP_1>
   #   grant_types: [client_credentials]
   #   scope: [mimir:read, mimir:write]
   
   # Apply updated config
   kubectl apply -f ygg-config.yaml
   
   # Restart Yggdrasil pod
   kubectl rollout restart deployment yggdrasil -n asgard
   kubectl wait --for=condition=ready pod -l app=yggdrasil -n asgard --timeout=60s
   ```

3. **Get Access Token**
   ```bash
   # Port-forward Yggdrasil
   kubectl port-forward svc/yggdrasil 9999:9999 -n asgard &
   
   curl -X POST http://localhost:9999/oauth/token \
     -H "Content-Type: application/x-www-form-urlencoded" \
     -d "grant_type=client_credentials&client_id=refgraph-s1&client_secret=$CLIENT_SECRET"
   ```

4. **Test with Mimir**
   ```bash
   # Port-forward Mimir (separate terminal)
   kubectl port-forward svc/mimir 5000:5000 -n asgard &
   
   export MIMIR_TOKEN="<access_token>"
   curl -X GET http://localhost:5000/api/v1/tenants/asgard_insurance/health \
     -H "Authorization: Bearer $MIMIR_TOKEN"
   ```

---

## Method 3: Via Yggdrasil REST API (Programmatic)

For automated client registration during CI/CD:

```bash
#!/bin/bash
# register-oauth2-client.sh

YGGDRASIL_URL="${YGGDRASIL_URL:-http://yggdrasil.asgard.internal}"
ADMIN_TOKEN="${ADMIN_TOKEN}"  # Must have admin scope

# Generate client secret
CLIENT_SECRET=$(openssl rand -hex 16)

# Register client
RESPONSE=$(curl -s -X POST "$YGGDRASIL_URL/api/v1/clients" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "refgraph-s1",
    "client_secret": "'$CLIENT_SECRET'",
    "grant_types": ["client_credentials"],
    "scope": ["mimir:read", "mimir:write"]
  }')

# Extract client ID from response
CLIENT_ID=$(echo $RESPONSE | jq -r '.client_id')
echo "Registered client: $CLIENT_ID"
echo "Client Secret: $CLIENT_SECRET"

# Save credentials
echo "export CLIENT_ID=$CLIENT_ID" > .env.oauth2
echo "export CLIENT_SECRET=$CLIENT_SECRET" >> .env.oauth2
chmod 600 .env.oauth2
```

---

## Troubleshooting

### Issue: 401 Unauthorized from Mimir

**Symptoms:**
```
POST /api/v1/tenants/asgard_insurance/ingest/entities HTTP/1.1
< HTTP/1.1 401 Unauthorized
< {"error": "missing or invalid authorization token"}
```

**Diagnosis:**

1. Check token exists
   ```bash
   echo $MIMIR_TOKEN
   # If empty: token not set
   ```

2. Check token format
   ```bash
   # Token should start with "eyJ" (base64-encoded JWT)
   echo $MIMIR_TOKEN | cut -c1-10
   ```

3. Check token not expired
   ```bash
   # Decode JWT (online: jwt.io)
   echo $MIMIR_TOKEN | cut -d'.' -f2 | base64 -d | jq '.exp'
   # Compare with current time: date +%s
   ```

4. Check Mimir can reach Yggdrasil
   ```bash
   # From Mimir pod/container, test Yggdrasil reachability
   curl -v http://yggdrasil.asgard.internal:9999/health
   # Should return 200 OK
   ```

**Solutions:**

- **Token Expired:** Get new token
  ```bash
  curl -X POST http://yggdrasil.asgard.internal/oauth/token \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=client_credentials&client_id=refgraph-s1&client_secret=$CLIENT_SECRET"
  ```

- **Client Not Registered:** Register via Method 1 or 2

- **Network Unreachable:** Check firewall rules and network routing
  ```bash
  # From your machine
  ping yggdrasil.asgard.internal
  curl -v http://yggdrasil.asgard.internal:9999/health
  ```

- **Port-Forward Issues:** Check port-forward is still active
  ```bash
  ps aux | grep "port-forward"
  # If missing, restart: kubectl port-forward svc/mimir 5000:5000 -n asgard &
  ```

### Issue: Client Registration Fails

**Symptoms:**
```
curl: (7) Failed to connect to yggdrasil.asgard.internal port 9999
```

**Solutions:**

1. Verify Yggdrasil is running
   ```bash
   kubectl get pod -l app=yggdrasil -n asgard
   kubectl logs yggdrasil-0 -n asgard --tail=20
   ```

2. Check network connectivity
   ```bash
   # From your machine
   docker run --rm -it alpine:latest sh -c "apk add curl && curl http://yggdrasil.asgard.internal:9999/health"
   ```

3. Use localhost with port-forward
   ```bash
   kubectl port-forward svc/yggdrasil 9999:9999 -n asgard &
   curl -v http://localhost:9999/health
   ```

### Issue: Asgard Tenant Not Found

**Symptoms:**
```
GET /api/v1/tenants/asgard_insurance/health
< HTTP/1.1 404 Not Found
< {"error": "tenant not found"}
```

**Solutions:**

1. Verify tenant exists
   ```bash
   # Query Mimir admin API (may require different auth)
   curl -X GET http://mimir.asgard.internal:5000/api/v1/tenants \
     -H "Authorization: Bearer $MIMIR_TOKEN"
   ```

2. Create tenant if missing (requires admin access)
   ```bash
   curl -X POST http://mimir.asgard.internal:5000/api/v1/tenants \
     -H "Authorization: Bearer $ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{
       "tenant_id": "asgard_insurance",
       "name": "Asgard Insurance",
       "domain": "insurance"
     }'
   ```

---

## Verify Setup

### Health Check Script

```bash
#!/bin/bash
# verify-oauth2-setup.sh

echo "=== OAuth2 + Real Mimir Verification ==="

# 1. Check environment variables
echo "[1/5] Environment Variables:"
if [ -z "$MIMIR_TOKEN" ]; then
  echo "  ❌ MIMIR_TOKEN not set"
  exit 1
fi
echo "  ✅ MIMIR_TOKEN set (${#MIMIR_TOKEN} chars)"

if [ -z "$MIMIR_URL" ]; then
  export MIMIR_URL="http://localhost:5000"
  echo "  ℹ️  MIMIR_URL not set, using default: $MIMIR_URL"
else
  echo "  ✅ MIMIR_URL = $MIMIR_URL"
fi

# 2. Check token format
echo "[2/5] Token Format:"
if [[ $MIMIR_TOKEN =~ ^eyJ ]]; then
  echo "  ✅ Token format valid (JWT)"
else
  echo "  ❌ Token format invalid (not JWT)"
  exit 1
fi

# 3. Check token expiration
echo "[3/5] Token Expiration:"
EXP=$(echo $MIMIR_TOKEN | cut -d'.' -f2 | base64 -d 2>/dev/null | jq -r '.exp' 2>/dev/null)
NOW=$(date +%s)
if [ "$EXP" -gt "$NOW" ]; then
  REMAINING=$((EXP - NOW))
  echo "  ✅ Token valid for $((REMAINING / 60)) minutes"
else
  echo "  ❌ Token expired"
  exit 1
fi

# 4. Check Mimir connectivity
echo "[4/5] Mimir Connectivity:"
HEALTH=$(curl -s -X GET "$MIMIR_URL/api/v1/tenants/asgard_insurance/health" \
  -H "Authorization: Bearer $MIMIR_TOKEN")

if echo "$HEALTH" | jq . > /dev/null 2>&1; then
  STATUS=$(echo "$HEALTH" | jq -r '.status' 2>/dev/null)
  echo "  ✅ Mimir healthy (status: $STATUS)"
else
  echo "  ❌ Mimir unreachable or invalid response"
  echo "  Response: $HEALTH"
  exit 1
fi

# 5. Check ingest endpoint
echo "[5/5] Ingest Endpoint:"
INGEST=$(curl -s -X POST "$MIMIR_URL/api/v1/tenants/asgard_insurance/ingest/entities" \
  -H "Authorization: Bearer $MIMIR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"entities": []}' \
  -w "\n%{http_code}" | tail -1)

if [ "$INGEST" = "200" ] || [ "$INGEST" = "201" ] || [ "$INGEST" = "202" ]; then
  echo "  ✅ Ingest endpoint accessible (HTTP $INGEST)"
else
  echo "  ❌ Ingest endpoint error (HTTP $INGEST)"
  exit 1
fi

echo ""
echo "✅ All checks passed! Ready to run May 23-28 data preparation."
```

Run it:
```bash
chmod +x verify-oauth2-setup.sh
./verify-oauth2-setup.sh
```

---

## Integration with May 23-28 Plan

### Before Starting Daily Tasks

```bash
# Day 1 (May 23) - Before any Prudential ingestion
source .env.oauth2  # If using env file from Method 3
export MIMIR_TOKEN="<your_token>"
export MIMIR_URL="http://mimir.asgard.internal:5000"  # or localhost:5000

# Verify setup
./verify-oauth2-setup.sh

# Then proceed with May 23 tasks
./target/release/orchestrate ingest prudential-docs.json \
  --mimir-url=$MIMIR_URL \
  --tenant-id=asgard_insurance \
  --verbose
```

### Token Refresh Strategy

For long-running sessions (May 23-28), tokens expire after 1 hour:

```bash
#!/bin/bash
# refresh-token.sh

CLIENT_ID="refgraph-s1"
CLIENT_SECRET="$1"  # Pass as argument
YGGDRASIL_URL="${YGGDRASIL_URL:-http://yggdrasil.asgard.internal}"

refresh_token() {
  local response=$(curl -s -X POST "$YGGDRASIL_URL/oauth/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=client_credentials&client_id=$CLIENT_ID&client_secret=$CLIENT_SECRET")
  
  export MIMIR_TOKEN=$(echo $response | jq -r '.access_token')
  echo "Token refreshed. Valid for $(echo $response | jq -r '.expires_in') seconds"
}

# Refresh if needed
refresh_token
```

Call before each daily task:
```bash
# May 23 morning
source .env.oauth2
./refresh-token.sh "$CLIENT_SECRET"

# Then run data preparation
./target/release/orchestrate ingest prudential-docs.json ...
```

---

## Next Steps

1. **Choose Setup Method:**
   - Method 1 (Admin Console): If you have manual access
   - Method 2 (K8s ConfigMap): If running on OrbStack locally
   - Method 3 (REST API): If automating in CI/CD

2. **Register Client:**
   - Follow steps for your chosen method
   - Save CLIENT_ID and CLIENT_SECRET securely

3. **Get Access Token:**
   - Use OAuth2 credentials to get JWT token
   - Set MIMIR_TOKEN environment variable

4. **Verify Setup:**
   - Run `verify-oauth2-setup.sh` to validate all components
   - Test sample ingest request

5. **Run Data Preparation:**
   - Proceed with May 23-28 daily tasks
   - Use token refresh script for multi-day execution

---

## Fallback: Continue Using Mock Server

If real Mimir OAuth2 setup is blocked:

```bash
# Keep using Python mock server (already validated)
python3 scripts/mock-mimir-server.py &
export MIMIR_URL="http://localhost:9999"
unset MIMIR_TOKEN  # Mock server doesn't require auth

# Run data preparation as before
./target/release/orchestrate ingest prudential-docs.json \
  --mimir-url=$MIMIR_URL \
  --tenant-id=asgard_insurance \
  --verbose
```

This is safe for May 23-28 validation. Real Mimir will be needed only for June 2-11 production execution.

---

## Summary

| Method | Setup Time | Auth Complexity | Recommended For |
|--------|-----------|-----------------|-----------------|
| Admin Console | 5 min | Low | Manual one-time setup |
| K8s ConfigMap | 10 min | Medium | Local OrbStack development |
| REST API | 15 min | High | Automated CI/CD |
| Mock Server | Already done | None | May 23-28 validation phase |

**Default for May 23-28:** Continue with mock server (no OAuth2 needed)  
**Required for June 2-11:** Real Mimir with OAuth2 client registration

---

**Status:** Complete step 3 ✅  
**Next:** Proceed to May 23-28 data preparation (May 23 morning)  
**Confidence:** 8.5/10 (actual OAuth2 setup depends on infrastructure access)
