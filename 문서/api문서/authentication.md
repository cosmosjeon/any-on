# Authentication Guide

Anyon uses GitHub OAuth for user authentication, specifically the Device Flow which is designed for CLI and desktop applications.

## Table of Contents

- [Overview](#overview)
- [GitHub OAuth Device Flow](#github-oauth-device-flow)
- [Implementation Guide](#implementation-guide)
- [Token Management](#token-management)
- [Personal Access Tokens](#personal-access-tokens)
- [Token Validation](#token-validation)

---

## Overview

Authentication in Anyon serves two purposes:

1. **Identity** - Identifying the user for analytics and Sentry error tracking
2. **GitHub API Access** - Enabling git push, PR creation, and GitHub API operations

**Authentication Methods:**
- **GitHub OAuth (Recommended)** - Device flow for easy authentication
- **Personal Access Token** - Manual token configuration

---

## GitHub OAuth Device Flow

The device flow is a user-friendly authentication method for applications that can't use a browser redirect.

### Flow Diagram

```
Client                          GitHub                         Anyon Server
  |                               |                                  |
  |-- POST /auth/github/device/start -------------------------------->|
  |<-- user_code, verification_uri, interval ------------------------|
  |                               |                                  |
  |-- Display code to user ------>|                                  |
  |                               |-- User enters code ------------->|
  |                               |<-- User authorizes app ----------|
  |                               |                                  |
  |-- POST /auth/github/device/poll (every 5s) --------------------->|
  |<-- "AUTHORIZATION_PENDING" -------------------------------------|
  |-- POST /auth/github/device/poll -------------------------------->|
  |<-- "SUCCESS" ---------------------------------------------------|
  |                               |                                  |
  |-- Use authenticated API -------------------------------------------->|
```

### Step 1: Start Device Flow

**Endpoint:** `POST /api/auth/github/device/start`

**Request:**
```bash
curl -X POST http://127.0.0.1:PORT/api/auth/github/device/start
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user_code": "ABCD-1234",
    "verification_uri": "https://github.com/login/device",
    "expires_in": 899,
    "interval": 5
  }
}
```

**Response Fields:**
- `user_code` - 8-character code to display to user
- `verification_uri` - URL where user should enter the code
- `expires_in` - Seconds until code expires (typically 15 minutes)
- `interval` - Minimum seconds between polling requests

### Step 2: Display Code to User

Show the `user_code` and `verification_uri` to the user:

```
Please visit: https://github.com/login/device
Enter code: ABCD-1234
```

The user will:
1. Open the verification URI in their browser
2. Enter the code
3. Authorize the Anyon application
4. Grant requested permissions

### Step 3: Poll for Authorization

**Endpoint:** `POST /api/auth/github/device/poll`

Start polling immediately after displaying the code to the user. Poll every `interval` seconds until you receive a non-pending response.

**Request:**
```bash
curl -X POST http://127.0.0.1:PORT/api/auth/github/device/poll
```

**Response (Still Waiting):**
```json
{
  "success": true,
  "data": "AUTHORIZATION_PENDING"
}
```

**Response (Polling Too Fast):**
```json
{
  "success": true,
  "data": "SLOW_DOWN"
}
```
When you receive `SLOW_DOWN`, increase your polling interval by 5 seconds.

**Response (Success):**
```json
{
  "success": true,
  "data": "SUCCESS"
}
```

When you receive `SUCCESS`, the OAuth token has been saved to the config file and you can now make authenticated API calls.

### Step 4: Token Storage

Upon successful authentication:

1. Token is automatically stored in `~/.anyon/config.json`
2. User information is retrieved from GitHub
3. Config is updated with:
   - `github.oauth_token` - OAuth access token
   - `github.username` - GitHub username
   - `github.primary_email` - User's primary email
   - `github_login_acknowledged` - Set to true

**Config File Location:**
- macOS/Linux: `~/.anyon/config.json`
- Windows: `%USERPROFILE%\.anyon\config.json`

---

## Implementation Guide

### JavaScript/TypeScript Example

```typescript
interface DeviceFlowResponse {
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

async function authenticateWithGitHub(): Promise<void> {
  // Step 1: Start device flow
  const startRes = await fetch('http://127.0.0.1:PORT/api/auth/github/device/start', {
    method: 'POST'
  });
  const startData = await startRes.json();
  const { user_code, verification_uri, interval } = startData.data as DeviceFlowResponse;

  // Step 2: Display code to user
  console.log(`Please visit: ${verification_uri}`);
  console.log(`Enter code: ${user_code}`);

  // Optionally auto-open browser
  // window.open(verification_uri, '_blank');

  // Step 3: Poll for authorization
  let pollInterval = interval * 1000; // Convert to milliseconds
  let authenticated = false;

  while (!authenticated) {
    await new Promise(resolve => setTimeout(resolve, pollInterval));

    const pollRes = await fetch('http://127.0.0.1:PORT/api/auth/github/device/poll', {
      method: 'POST'
    });
    const pollData = await pollRes.json();

    switch (pollData.data) {
      case 'SUCCESS':
        console.log('Authentication successful!');
        authenticated = true;
        break;
      case 'SLOW_DOWN':
        console.log('Slowing down polling...');
        pollInterval += 5000; // Add 5 seconds
        break;
      case 'AUTHORIZATION_PENDING':
        console.log('Waiting for user authorization...');
        break;
      default:
        console.error('Unexpected response:', pollData);
    }
  }
}
```

### Python Example

```python
import requests
import time

def authenticate_with_github():
    base_url = "http://127.0.0.1:PORT/api"

    # Step 1: Start device flow
    start_res = requests.post(f"{base_url}/auth/github/device/start")
    start_data = start_res.json()["data"]

    user_code = start_data["user_code"]
    verification_uri = start_data["verification_uri"]
    interval = start_data["interval"]

    # Step 2: Display code
    print(f"Please visit: {verification_uri}")
    print(f"Enter code: {user_code}")

    # Step 3: Poll for authorization
    poll_interval = interval
    authenticated = False

    while not authenticated:
        time.sleep(poll_interval)

        poll_res = requests.post(f"{base_url}/auth/github/device/poll")
        status = poll_res.json()["data"]

        if status == "SUCCESS":
            print("Authentication successful!")
            authenticated = True
        elif status == "SLOW_DOWN":
            print("Slowing down polling...")
            poll_interval += 5
        elif status == "AUTHORIZATION_PENDING":
            print("Waiting for authorization...")
        else:
            print(f"Unexpected response: {status}")
            break

authenticate_with_github()
```

### Rust Example

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize)]
struct DeviceFlowResponse {
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
}

async fn authenticate_with_github() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let base_url = "http://127.0.0.1:PORT/api";

    // Step 1: Start device flow
    let start_res: ApiResponse<DeviceFlowResponse> = client
        .post(&format!("{}/auth/github/device/start", base_url))
        .send()
        .await?
        .json()
        .await?;

    let flow = start_res.data.unwrap();

    // Step 2: Display code
    println!("Please visit: {}", flow.verification_uri);
    println!("Enter code: {}", flow.user_code);

    // Step 3: Poll for authorization
    let mut poll_interval = Duration::from_secs(flow.interval);

    loop {
        sleep(poll_interval).await;

        let poll_res: ApiResponse<String> = client
            .post(&format!("{}/auth/github/device/poll", base_url))
            .send()
            .await?
            .json()
            .await?;

        match poll_res.data.as_deref() {
            Some("SUCCESS") => {
                println!("Authentication successful!");
                break;
            }
            Some("SLOW_DOWN") => {
                println!("Slowing down polling...");
                poll_interval += Duration::from_secs(5);
            }
            Some("AUTHORIZATION_PENDING") => {
                println!("Waiting for authorization...");
            }
            _ => {
                eprintln!("Unexpected response");
                break;
            }
        }
    }

    Ok(())
}
```

---

## Token Management

### Token Permissions

The GitHub OAuth token requests the following scopes:
- `repo` - Full repository access (read/write)
- `user` - Read user profile information

### Token Validation

**Endpoint:** `GET /api/auth/github/check`

**Description:** Check if the stored GitHub token is still valid.

**Request:**
```bash
curl http://127.0.0.1:PORT/api/auth/github/check
```

**Response (Valid):**
```json
{
  "success": true,
  "data": "VALID"
}
```

**Response (Invalid):**
```json
{
  "success": true,
  "data": "INVALID"
}
```

**Use Cases:**
- Check token validity before making GitHub API calls
- Prompt user to re-authenticate if token is invalid
- Periodic health checks

### Token Refresh

GitHub OAuth tokens do not expire automatically, but they can be:
- Revoked by the user on GitHub
- Invalidated if app permissions change
- Deleted if the OAuth app is removed

If a token becomes invalid, the user must re-authenticate using the device flow.

---

## Personal Access Tokens

For users who prefer manual configuration, Anyon supports GitHub Personal Access Tokens (PAT).

### Creating a PAT

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Give it a descriptive name (e.g., "Anyon CLI")
4. Select scopes:
   - `repo` (Full control of private repositories)
   - `user` (Read user profile data)
5. Click "Generate token"
6. Copy the token immediately (it won't be shown again)

### Configuring PAT

Manually edit `~/.anyon/config.json`:

```json
{
  "github": {
    "pat": "ghp_YOUR_TOKEN_HERE",
    "oauth_token": null,
    "username": "your-username",
    "primary_email": "you@example.com",
    "default_pr_base": "main"
  }
}
```

**OR** use the API:

```bash
curl -X PUT http://127.0.0.1:PORT/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "github": {
      "pat": "ghp_YOUR_TOKEN_HERE",
      "username": "your-username",
      "primary_email": "you@example.com"
    },
    ... // other config fields
  }'
```

### PAT vs OAuth

| Feature | PAT | OAuth |
|---------|-----|-------|
| User Experience | Manual setup | Automated flow |
| Expiration | Can be set | No expiration |
| Revocation | Manual on GitHub | Manual on GitHub |
| Scopes | User-selected | App-defined |
| Security | User-managed | App-managed |

**Recommendation:** Use OAuth for better UX unless you have specific security requirements.

---

## Token Usage

Once authenticated, the token is automatically used for:

### Git Operations

- `git push` to GitHub
- `git fetch` from private repositories
- `git clone` of private repositories

### GitHub API Calls

- Creating pull requests
- Listing repositories
- Checking PR status
- Attaching existing PRs
- Getting repository information

### Authentication Header

The token is sent as:

```
Authorization: Bearer {token}
```

Or for git operations:

```
https://{token}@github.com/owner/repo.git
```

---

## Security Best Practices

1. **Never commit tokens** - Keep config files out of version control
2. **Use minimal scopes** - Only request necessary permissions
3. **Rotate tokens periodically** - Generate new PATs every few months
4. **Revoke unused tokens** - Clean up old tokens on GitHub
5. **Protect config files** - Ensure config files have restricted permissions

### Config File Permissions

On macOS/Linux:
```bash
chmod 600 ~/.anyon/config.json
```

This ensures only the owner can read/write the config file.

---

## Error Handling

### Common Errors

**Invalid Token:**
```json
{
  "success": false,
  "error_data": {
    "TOKEN_INVALID"
  },
  "message": "GitHub token is invalid or expired"
}
```

**Insufficient Permissions:**
```json
{
  "success": false,
  "error_data": {
    "INSUFFICIENT_PERMISSIONS"
  },
  "message": "Token lacks required permissions"
}
```

**Repository Not Found:**
```json
{
  "success": false,
  "error_data": {
    "REPO_NOT_FOUND_OR_NO_ACCESS"
  },
  "message": "Repository not found or token has no access"
}
```

### Error Recovery

1. **Invalid Token** - Re-authenticate using device flow
2. **Insufficient Permissions** - Generate new token with correct scopes
3. **Repo Access** - Ensure token has access to the repository

---

## Testing Authentication

### Manual Test

```bash
# 1. Start device flow
curl -X POST http://127.0.0.1:PORT/api/auth/github/device/start

# 2. Authorize on GitHub (use returned code)

# 3. Poll until success
while true; do
  curl -X POST http://127.0.0.1:PORT/api/auth/github/device/poll
  sleep 5
done

# 4. Verify token
curl http://127.0.0.1:PORT/api/auth/github/check
```

### Automated Test

Use the code examples above to build an automated test that:
1. Starts the device flow
2. Displays the code
3. Waits for manual authorization
4. Verifies successful authentication
5. Tests a GitHub API call

---

## Troubleshooting

### Device Flow Not Starting

**Symptom:** `/auth/github/device/start` returns error

**Possible Causes:**
- Invalid GitHub client ID
- Network connectivity issues
- GitHub API unavailable

**Solution:**
1. Check network connection
2. Verify `GITHUB_CLIENT_ID` environment variable
3. Check GitHub status: https://www.githubstatus.com/

### Polling Never Succeeds

**Symptom:** `/auth/github/device/poll` always returns `AUTHORIZATION_PENDING`

**Possible Causes:**
- User hasn't authorized the app
- Code expired (15 minutes)
- User entered wrong code

**Solution:**
1. Verify user completed authorization on GitHub
2. Check code hasn't expired
3. Restart flow if needed

### Token Invalid After Setup

**Symptom:** Token works initially but becomes invalid

**Possible Causes:**
- Token revoked by user on GitHub
- Token expired (PAT only)
- OAuth app deleted

**Solution:**
1. Check GitHub token status in settings
2. Re-authenticate using device flow
3. Generate new PAT if using manual token

---

## Next Steps

After authentication, you can:

1. **Create Projects** - Set up git repositories
2. **Create Tasks** - Define work items
3. **Start Task Attempts** - Run AI coding agents
4. **Push to GitHub** - Share your changes
5. **Create PRs** - Collaborate with your team

See [endpoints.md](./endpoints.md) for API details.
