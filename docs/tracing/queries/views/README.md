# Athena Views — Setup Order

Run these once in the Athena console under workgroup **trellislab-analytics**, database **cf_logs**.
Re-run any view to update its definition (they use `CREATE OR REPLACE VIEW`).

## Prerequisites

1. Phase 1 complete: CloudFront logs landing in `s3://trellislab-cloudfront-logs/cf-logs/`
2. Phase 2 complete: `cf_logs.access_logs_raw` table exists and returns rows
3. PoP reference data uploaded:
   ```bash
   aws s3 cp docs/tracing/data/pop_to_country.csv \
     s3://trellislab-cloudfront-logs/ref-data/pop_to_country.csv \
     --profile trellislab
   ```
4. Phase 3 Terraform applied (`analytics_enrichment.tf`) so `cf_logs.pop_to_country` table exists

## Execution order

| # | File | Creates |
|---|------|---------|
| 1 | `01_requests_enriched.sql` | `cf_logs.requests_enriched` — base enriched view (geo, UA, visitor hash, timestamp) |
| 2 | `02_sessions.sql` | `cf_logs.sessions` — one row per visitor per day |
| 3 | `03_daily_summary.sql` | `cf_logs.daily_summary` — one row per day, all KPIs |

## Verify

```sql
-- Should return rows if logs exist
SELECT * FROM cf_logs.requests_enriched
WHERE request_date = current_date - interval '1' day
LIMIT 10;

-- Top countries yesterday
SELECT country_name, COUNT(*) AS hits
FROM cf_logs.requests_enriched
WHERE request_date = current_date - interval '1' day
  AND browser_family <> 'bot'
GROUP BY country_name
ORDER BY hits DESC
LIMIT 10;

-- Daily unique visitors
SELECT request_date, unique_visitors, sessions, human_requests
FROM cf_logs.daily_summary
ORDER BY request_date DESC
LIMIT 14;
```

## Salt rotation

The `visitor_hash` in `requests_enriched` uses a static salt string embedded in the view.
If you rotate the salt, re-run `01_requests_enriched.sql` with the new value.
Keep the salt outside the repo (e.g. AWS Secrets Manager) and substitute at view creation time
once visitor counts become meaningful for privacy compliance.
