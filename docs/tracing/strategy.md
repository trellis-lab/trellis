# Visitor Tracking Strategy — trellislab.net

**Date:** 2026-04-26
**Scope:** Server-side visitor analytics for the static site
**Chosen approach:** Option 4 — CloudFront access logs → S3 → Athena → Grafana/QuickSight
**Why:** 100% capture (no ad-blocker blind spots), GDPR-friendly (hash IPs at query time), no client-side JS, cheap at low traffic, fits existing AWS-native infra in [setup-infra/strategy.md](../setup-infra/strategy.md).

---

## 1. Target Outcomes

| Question | Phase that answers it |
|---|---|
| How many visits per day, per page? | Phase 2 |
| Which countries / cities visit? | Phase 2 |
| Which referrers / campaigns drive traffic? | Phase 2 |
| Which browsers / OS / devices? | Phase 3 |
| What is the WASM payload hit/miss ratio? | Phase 3 |
| How many unique visitors / sessions per day? | Phase 3 |
| Bot vs human traffic split? | Phase 4 |
| 4xx/5xx errors, slow paths? | Phase 4 |

---

## 2. Phased Plan Overview

| Phase | Goal | Effort | Cost delta |
|---|---|---|---|
| **0** | Decision + this doc | done | — |
| **1** | Enable CloudFront logging → S3 (collect now, query later) | 0.5 day | ~$0.05/mo |
| **2** | Athena table + first SQL queries (manual, ad-hoc) | 0.5 day | ~$0.10/mo |
| **3** | Geo + UA enrichment, sessionisation views | 1 day | ~$0.10/mo |
| **4** | Local Grafana instance reading from AWS (or manual CSV) | 0.5 day | $0 |
| **5** | Optional: QuickSight or Grafana Cloud for shared dashboards | 0.5 day | $9/mo (QuickSight) or free tier (Grafana Cloud) |
| **6** | Optional: Real-time logs via Kinesis (if needed) | 1 day | ~$5/mo |
| **7** | Lifecycle policies, log compaction, partition projection | 0.5 day | saves cost long-term |

Phases 1–4 deliver a working analytics pipeline. Phases 5–7 are upgrades.

---

## 3. Phase 1 — Enable CloudFront Logging

**Goal:** Start collecting raw access logs immediately. Data accumulates from day one even if dashboards come later.

### What to build

- New S3 bucket `trellislab-cloudfront-logs` in same region as CloudFront log delivery (CloudFront standard logs are written to any S3 bucket).
- Block public access, AES-256 encryption, lifecycle rule (see Phase 7).
- Bucket policy granting `awslogsdelivery` account write access (CloudFront standard logs use legacy ACLs — bucket must allow `BucketOwnerPreferred` ACL or use the newer V2 logging which uses bucket policy).
- Update CloudFront distribution: enable standard logging, prefix `cf-logs/`, include cookies = `false`.

### Terraform sketch

```hcl
# terraform/environments/prod/logs.tf
resource "aws_s3_bucket" "cf_logs" {
  bucket = "trellislab-cloudfront-logs"
}

resource "aws_s3_bucket_ownership_controls" "cf_logs" {
  bucket = aws_s3_bucket.cf_logs.id
  rule { object_ownership = "BucketOwnerPreferred" }
}

resource "aws_s3_bucket_acl" "cf_logs" {
  depends_on = [aws_s3_bucket_ownership_controls.cf_logs]
  bucket     = aws_s3_bucket.cf_logs.id
  acl        = "log-delivery-write"
}

# in the existing aws_cloudfront_distribution:
logging_config {
  bucket          = aws_s3_bucket.cf_logs.bucket_domain_name
  prefix          = "cf-logs/"
  include_cookies = false
}
```

### Verify

- After ~15 min, list `s3://trellislab-cloudfront-logs/cf-logs/`.
- Files: `<distribution-id>.YYYY-MM-DD-HH.<hash>.gz`.
- Each line is tab-separated: see [CloudFront standard log format](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/AccessLogs.html#LogFileFormat).

### Exit criteria

- Logs visible in S3.
- One full day of data archived before moving to Phase 2.

---

## 4. Phase 2 — Athena Table + Ad-hoc Queries

**Goal:** Query the raw logs with SQL. No dashboards yet — just answer questions on demand.

### What to build

- Athena workgroup `trellislab-analytics` (separate workgroup → separate cost tracking, separate query result location).
- Athena query result S3 bucket: `trellislab-athena-results` with 30-day lifecycle deletion.
- External table `cf_logs.access_logs_raw` over `s3://trellislab-cloudfront-logs/cf-logs/` using the [official CloudFront DDL](https://docs.aws.amazon.com/athena/latest/ug/cloudfront-logs.html).
- Date-partitioned view: `cf_logs.access_logs` with `year/month/day` extracted from the timestamp. Use **partition projection** (no Glue crawler, no `MSCK REPAIR`).

### DDL sketch

```sql
CREATE EXTERNAL TABLE IF NOT EXISTS cf_logs.access_logs_raw (
  request_date date,
  request_time string,
  edge_location string,
  bytes_sent bigint,
  client_ip string,
  http_method string,
  cs_host string,
  uri_stem string,
  status int,
  referer string,
  user_agent string,
  uri_query string,
  cookie string,
  edge_result_type string,
  edge_request_id string,
  host_header string,
  protocol string,
  cs_bytes bigint,
  time_taken double,
  forwarded_for string,
  ssl_protocol string,
  ssl_cipher string,
  edge_response_result_type string,
  protocol_version string,
  fle_status string,
  fle_encrypted_fields int,
  c_port int,
  time_to_first_byte double,
  edge_detailed_result_type string,
  sc_content_type string,
  sc_content_len bigint,
  sc_range_start bigint,
  sc_range_end bigint
)
ROW FORMAT DELIMITED FIELDS TERMINATED BY '\t'
LOCATION 's3://trellislab-cloudfront-logs/cf-logs/'
TBLPROPERTIES ('skip.header.line.count'='2');
```

### First queries (save in workgroup)

```sql
-- Daily request count
SELECT request_date, COUNT(*) AS hits
FROM cf_logs.access_logs_raw
WHERE request_date >= current_date - interval '7' day
GROUP BY request_date ORDER BY 1;

-- Top URIs
SELECT uri_stem, COUNT(*) AS hits
FROM cf_logs.access_logs_raw
WHERE request_date = current_date - interval '1' day
GROUP BY uri_stem ORDER BY hits DESC LIMIT 20;

-- Top referers (excluding self)
SELECT referer, COUNT(*) AS hits
FROM cf_logs.access_logs_raw
WHERE request_date >= current_date - interval '7' day
  AND referer NOT LIKE '%trellislab.net%'
  AND referer <> '-'
GROUP BY referer ORDER BY hits DESC LIMIT 20;
```

### Exit criteria

- Five canned queries saved in the workgroup.
- One real question answered (e.g. "what was traffic last week?").

---

## 5. Phase 3 — Enrichment: Geo, UA Parsing, Sessions

**Goal:** Promote raw logs into something usable for dashboards. Build views, not new tables.

### Geo lookup (country)

Two paths — pick one:

**5a. MaxMind GeoLite2 (free, accurate, ~3M rows)**
1. Download `GeoLite2-Country-CSV` from maxmind.com (free account).
2. Convert IP-range CIDR to integer ranges; upload to S3 as Parquet.
3. Athena view joins `inet_aton(client_ip)` to the range table.

**5b. AWS-only quick path (less accurate but zero deps)**
- Use the CloudFront `edge_location` field — first 3 letters are an IATA-style PoP code (e.g. `FRA50` → Frankfurt → DE).
- Maintain a small static `pop_to_country` lookup table in Athena.
- Coarser (you get **edge** location not **viewer** location), but no external data.

Recommendation: start with 5b for speed; upgrade to 5a if accuracy matters.

### UA parsing

Athena has no built-in UA parser. Options:
- Regex-based view (browser families: Chrome, Firefox, Safari, Edge, bot).
- AWS supports [Athena UDFs via Lambda](https://docs.aws.amazon.com/athena/latest/ug/querying-udf.html) — overkill at this stage.

Start with regex view:

```sql
CREATE OR REPLACE VIEW cf_logs.requests_enriched AS
SELECT
  *,
  CASE
    WHEN regexp_like(user_agent, '(?i)bot|crawler|spider|curl|wget') THEN 'bot'
    WHEN regexp_like(user_agent, 'Edg/')      THEN 'Edge'
    WHEN regexp_like(user_agent, 'Chrome/')   THEN 'Chrome'
    WHEN regexp_like(user_agent, 'Firefox/')  THEN 'Firefox'
    WHEN regexp_like(user_agent, 'Safari/')   THEN 'Safari'
    ELSE 'other'
  END AS browser_family,
  CASE
    WHEN regexp_like(user_agent, 'Mobile|Android|iPhone') THEN 'mobile'
    WHEN regexp_like(user_agent, 'Tablet|iPad')           THEN 'tablet'
    ELSE 'desktop'
  END AS device_class
FROM cf_logs.access_logs_raw;
```

### Sessionisation

Define a session as: same `client_ip` + `user_agent` with no gap > 30 min.

```sql
CREATE OR REPLACE VIEW cf_logs.sessions AS
SELECT
  client_ip,
  user_agent,
  request_date,
  COUNT(*) AS hits,
  MIN(parse_datetime(...)) AS started_at,
  MAX(parse_datetime(...)) AS last_hit
FROM cf_logs.requests_enriched
WHERE browser_family <> 'bot'
GROUP BY client_ip, user_agent, request_date;
```

(More accurate sessionisation uses `LAG()` to detect 30-min gaps — defer until needed.)

### Privacy

GDPR: hash `client_ip` in any persisted/exported dataset.

```sql
SELECT to_hex(sha256(to_utf8(client_ip || 'salt'))) AS visitor_hash, ...
```

### Exit criteria

- Three views in `cf_logs` schema: `requests_enriched`, `sessions`, `daily_summary`.
- "Top countries" and "browser breakdown" answerable in one query.

---

## 6. Phase 4 — Local Grafana Reading from AWS

**Goal:** Visualisations on the local laptop. No QuickSight cost. Two paths.

### 6a. Live mode — Grafana → Athena via plugin (preferred)

Grafana has an official **Amazon Athena data source** plugin. Runs queries against the same Athena workgroup. Charts auto-refresh.

**Setup**
1. `docker run -d -p 3000:3000 --name grafana grafana/grafana-oss:latest`
2. Install the Athena plugin: `grafana-cli plugins install grafana-athena-datasource` (or use the `grafana/grafana` image with `GF_INSTALL_PLUGINS=grafana-athena-datasource`).
3. Create IAM user `grafana-readonly` with policy:
   - `athena:StartQueryExecution`, `GetQueryResults`, `GetQueryExecution` on workgroup `trellislab-analytics`
   - `s3:GetObject`, `ListBucket` on `trellislab-cloudfront-logs` and `trellislab-athena-results`
   - `glue:GetTable`, `GetDatabase`, `GetPartitions` on `cf_logs` database
4. Generate access keys; configure Grafana data source (region `eu-central-1`, workgroup `trellislab-analytics`, database `cf_logs`).
5. Build dashboards: pageviews, geo map, top URIs, browser pie, session count.

**docker-compose.yml** (commit to repo under `docs/tracing/grafana/`):

```yaml
services:
  grafana:
    image: grafana/grafana-oss:latest
    ports: ["3000:3000"]
    environment:
      GF_INSTALL_PLUGINS: grafana-athena-datasource
      GF_AUTH_ANONYMOUS_ENABLED: "false"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./provisioning:/etc/grafana/provisioning
volumes:
  grafana-data:
```

Each query you run in Grafana costs ~$5/TB scanned by Athena → at our volumes pennies per month. Use partition pruning (`WHERE request_date >= ...`) on every panel.

**Cost guard:** set `--max-query-cost` via Athena workgroup setting (`Per query data limit`, e.g. 100 MB). Caps any runaway query.

### 6b. Manual download mode — fallback

When you want to work offline or avoid even the Grafana-IAM-user setup:

1. Run the canned Athena queries via console / CLI.
2. Athena writes CSV results to `s3://trellislab-athena-results/...`.
3. Download CSVs locally:
   ```bash
   aws s3 sync s3://trellislab-athena-results/Unsaved/2026/04/26/ ./local-data/
   ```
4. Use Grafana's CSV data source plugin (`marcusolsson-csv-datasource`) to chart from local files.
5. Or load into **DuckDB** locally (`duckdb`, `read_csv_auto('local-data/*.csv')`) and use the Grafana DuckDB plugin / Metabase.

This path needs zero AWS credentials inside Grafana.

### Exit criteria

- Local Grafana running on `http://localhost:3000`.
- Dashboard with at least: daily pageviews, top 10 URIs, top 10 countries (or PoPs), browser breakdown, status code distribution.
- Manual CSV path documented in this folder for offline use.

---

## 7. Phase 5 — Optional: Shared Dashboards

Skip until needed. Two cheap routes:

| Tool | Cost | Auth | Notes |
|---|---|---|---|
| QuickSight Standard | $9/user/mo | AWS IAM | Native Athena integration, polished |
| Grafana Cloud free | $0 | Grafana auth | 10k series, 14-day retention; point at Athena via plugin |
| Self-host Grafana on EC2 t4g.nano + Caddy | ~$3/mo | Reverse-proxy basic auth | Same as local but exposed at `analytics.trellislab.net` |

Recommendation: **Grafana Cloud free** if it must be shared. EC2-hosted Grafana if you outgrow free tier.

---

## 8. Phase 6 — Optional: Real-time Logs (Kinesis)

Standard logs lag 5–15 min. If you need < 1 min latency (e.g. for a launch event):

- Enable CloudFront real-time logs → Kinesis Data Stream → S3 (Firehose) and/or OpenSearch.
- Cost: Kinesis ~$0.014/shard-hour ≈ $10/mo per shard + $0.014 per 1M events.
- Defer until there is a concrete need. Standard logs are sufficient for dashboards.

---

## 9. Phase 7 — Cost & Maintenance Hardening

Once volume grows:

- **Lifecycle on log bucket:** transition `cf-logs/` to `STANDARD_IA` after 30 days, `GLACIER_IR` after 90 days, expire after 365.
- **Partition projection** on Athena table — eliminates need for `MSCK REPAIR` / Glue crawler. Update DDL:
  ```sql
  TBLPROPERTIES (
    'projection.enabled' = 'true',
    'projection.year.type' = 'integer', 'projection.year.range' = '2026,2030',
    ...
    'storage.location.template' = 's3://.../cf-logs/${year}/${month}/${day}/'
  )
  ```
  Note: CloudFront standard logs are **not** date-prefixed by default. Either move to V2 logging (which supports custom prefix template), or run a small daily Lambda to rename `<dist>.YYYY-MM-DD-HH.*` files into `year=YYYY/month=MM/day=DD/` prefixes. Saves 95% of Athena scan cost.
- **Compact to Parquet:** monthly Glue/Lambda job converts daily logs from gzip-TSV to columnar Parquet partitioned by date. Athena queries 10–50× cheaper. Worth doing when monthly logs exceed 1 GB.
- **Athena workgroup data scan limit:** enforce `100 MB / query` to prevent runaway dashboards.

---

## 10. Privacy & Compliance Checklist

- [ ] Cookies disabled in CloudFront log config (`include_cookies = false`).
- [ ] No PII (forms, user IDs) sent in URLs.
- [ ] IPs hashed with salt before any export from Athena.
- [ ] Privacy policy page mentions: server-side logs, retention period, purpose.
- [ ] No third-party trackers loaded — site stays cookie-banner-free under GDPR ePrivacy.
- [ ] Logs bucket: SSE-S3 encryption, public access blocked, versioning off (logs are append-only & cheap to recreate).

---

## 11. File / Resource Inventory (end state)

```
AWS:
  S3:        trellislab-cloudfront-logs        (raw .gz logs)
             trellislab-athena-results         (query CSVs, 30-day TTL)
  Glue DB:   cf_logs
  Glue tables: access_logs_raw
  Glue views:  requests_enriched, sessions, daily_summary
  Athena WG: trellislab-analytics              (100 MB/query cap)
  IAM user:  grafana-readonly                  (athena+s3+glue read)

Repo:
  terraform/environments/prod/logs.tf          (S3 + CloudFront logging_config)
  terraform/environments/prod/analytics.tf     (Athena WG, Glue DB/table, IAM)
  docs/tracing/strategy.md                     (this file)
  docs/tracing/queries/*.sql                   (canned Athena queries)
  docs/tracing/grafana/docker-compose.yml      (local Grafana)
  docs/tracing/grafana/provisioning/           (data sources + dashboards as JSON)
```

---

## 12. Decision Log

| Decision | Choice | Rationale |
|---|---|---|
| Tracking method | CloudFront logs → Athena | 100% capture, GDPR-friendly, no JS, fits AWS-native infra |
| Geo lookup | CloudFront PoP first, MaxMind later | Coarse but free; upgrade only when needed |
| UA parsing | Athena regex view | Sufficient at current scale; UDF overkill |
| Dashboard tool | Local Grafana OSS via Athena plugin | $0, runs on laptop, same SQL as prod |
| Fallback path | Athena CSV download → DuckDB / Grafana CSV plugin | Works offline, no AWS keys in Grafana |
| Real-time logs | Deferred (Phase 6) | $10+/mo not justified pre-launch |
| QuickSight | Deferred (Phase 5, optional) | Adds $9/user/mo; not needed solo |
| Vendor lock | Accepted (low priority) | All data is plain TSV in S3 — exit cost is just `aws s3 sync` |

---

## 13. Next Action

Phase 1: write `terraform/environments/prod/logs.tf`, plan, apply. Verify logs land in S3 within 15 min.
