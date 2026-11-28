# Self-Hosting Keeftalk Desktop

Keeftalk Desktop can connect to any self-hosted [Keeftalk Backend](https://github.com/Keeftalk/Backend) (Supabase + Cloudflare R2) without using hosted Keeftalk servers.

## 1. Deploy Your Backend
1. Provision a Supabase project (or self-hosted Supabase via Docker) and apply the schema/migrations from `Keeftalk/Backend`.
2. Deploy the Supabase Edge Functions (`storage`, `billing-*`, `two-factor-auth`, `link-preview`).
3. Configure your Cloudflare R2 (or S3-compatible) bucket secrets in Supabase Edge Function environment variables.

## 2. Point Keeftalk Desktop to Your Instance
Configure via environment variables or your OS `config.toml`:

```bash
export KEEFTALK_MODE=cloud
export KEEFTALK_SUPABASE_URL="https://your-supabase-instance.example.com"
export KEEFTALK_SUPABASE_ANON_KEY="your-public-anon-key"
```

Or in `config.toml` (see `config/development.example.toml`):
```toml
[app]
mode = "cloud"

[backend]
supabase_url = "https://your-supabase-instance.example.com"
supabase_anon_key = "your-public-anon-key"
```

Never place a `service_role` key in any desktop configuration or binary.
