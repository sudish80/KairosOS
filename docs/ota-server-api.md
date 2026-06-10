# KairosOS OTA Update Server API v1

Base URL: `https://updates.kairosos.org/v1`

## Endpoints

### `GET /check`

Check for available updates.

**Query Parameters:**
| Param | Type | Description |
|-------|------|-------------|
| `device_id` | string | Unique device identifier (machine-id or UUID) |
| `channel` | string | Update channel: `stable`, `beta`, `nightly` |
| `version` | string | Current software version (e.g. `1.0.0`) |

**Response (200):** `UpdateManifest` JSON (see schema below)

**Response (204):** No update available

### `GET /download/<release_id>/<filename>`

Download an update image or delta patch.

**Headers:**
| Header | Value |
|--------|-------|
| `X-Kairos-Device-Id` | Device identifier |
| `X-Kairos-Channel` | Update channel |

**Response (200):** Binary stream (image or patch file)

### `POST /report`

Report update status back to server.

**Request Body:**
```json
{
  "device_id": "string",
  "release_id": "string",
  "state": "downloaded|applied|failed|rolled_back",
  "error": "string (optional)",
  "previous_version": "string",
  "new_version": "string",
  "timestamp": "ISO8601"
}
```

**Response (200):** `{"status": "ok"}`

---

## Update Manifest Schema

```json
{
  "format_version": 1,
  "release_id": "2026-06-10-001",
  "version": "1.0.1",
  "channel": "stable",
  "staging_percentage": 10,
  "images": [
    {
      "target": "full",
      "url": "https://updates.kairosos.org/v1/download/2026-06-10-001/kairosos-1.0.1.img.gz",
      "sha256": "abcdef0123456789...",
      "size_bytes": 2147483648,
      "compression": "gzip"
    }
  ],
  "deltas": [
    {
      "from_version": "1.0.0",
      "to_version": "1.0.1",
      "url": "https://updates.kairosos.org/v1/download/2026-06-10-001/delta-1.0.0-1.0.1.bspatch",
      "sha256": "fedcba9876543210...",
      "size_bytes": 52428800
    }
  ],
  "min_bootloader_version": "2026.02",
  "signing_key_fingerprint": "A1B2C3D4E5F6...",
  "timestamp": "2026-06-10T00:00:00Z",
  "description": "Security fix: patched buffer overflow in BPF handler"
}
```

## Staged Rollout

The client determines eligibility by hashing its `device_id`:
- `SHA256(device_id)` → first 4 bytes as u32 → `hash % 100`
- Device is eligible if `hash_pct < staging_percentage`
- Server can also override by returning `eligible: false` in `/check`

## Rollback Strategy

1. Max 3 boot attempts per slot (configurable)
2. On 3rd failure: automatic rollback to previous slot
3. Failed slot marked `bad` — not attempted again
4. Manual rollback via `ota-update.sh rollback` or `kairos-recovery --rollback`
