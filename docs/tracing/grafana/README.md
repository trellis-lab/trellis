# Local Grafana — Phase 4 Setup

Analytics dashboard running locally, querying Athena live via the AWS Athena datasource plugin.

## Prerequisites

- Docker + Docker Compose
- Phase 2 and 3 Terraform applied (`analytics.tf`, `analytics_enrichment.tf`)
- Athena views created (run SQL in `docs/tracing/queries/views/` in order)
- IAM user `grafana-readonly` created by Terraform

## 1 — Create IAM access key

```
AWS Console → IAM → Users → grafana-readonly
  → Security credentials → Create access key
  → Use case: Application running outside AWS
```

Copy the key ID and secret. Shown only once.

## 2 — Create .env

```bash
cp docs/tracing/grafana/.env.example docs/tracing/grafana/.env
# fill in real key values
```

The `.env` file is gitignored — never commit it.

## 3 — Start Grafana

```bash
cd docs/tracing/grafana
docker compose up -d
```

First start downloads and installs the `grafana-athena-datasource` plugin (~30 sec).
Open: http://localhost:3000  
Default login: `admin` / `admin` (change on first login).

## 4 — Verify datasource

Grafana → Connections → Data sources → Athena → **Test**

If test passes, the provisioned dashboard loads automatically:  
Grafana → Dashboards → **Trellis Analytics**

## 5 — Using the dashboard

- Time range picker (top right): adjust to your available log range.
- All panels use `$__dateFilter(request_date)` — Athena only scans the
  selected date range (cheap if using the partitioned `access_logs` table).
- Each panel query runs independently; results are cached by Grafana for 1 h.

### Cost guard

Every panel query hits Athena. At current log volume (~MB/day), each query
scans < 1 MB — cost is negligible. The workgroup enforces a hard 100 MB cap
per query. If you extend date ranges to months, monitor the Athena cost tab.

---

## Manual CSV fallback (offline / no IAM setup)

If you want dashboards without AWS credentials in Grafana:

### Step 1 — Export CSVs from Athena console

Run any query from `docs/tracing/queries/`. Athena writes results to:
```
s3://trellislab-athena-results/results/<query-id>.csv
```

Download locally:
```bash
# List recent result files
aws s3 ls s3://trellislab-athena-results/results/ --profile trellislab | tail -20

# Sync a specific query result
aws s3 cp s3://trellislab-athena-results/results/<query-id>.csv ./local-data/ --profile trellislab

# Or bulk-sync today's results
aws s3 sync \
  "s3://trellislab-athena-results/results/" \
  ./local-data/ \
  --exclude "*" --include "*.csv" \
  --profile trellislab
```

### Step 2 — Load in Grafana via CSV plugin

Add to `docker-compose.yml` environment:
```yaml
GF_INSTALL_PLUGINS: grafana-athena-datasource,marcusolsson-csv-datasource
```

Then add a new datasource:
- Type: **CSV**
- Path: `/var/lib/grafana/csv` (mount `./local-data` to that path in compose)

### Step 3 — Or use DuckDB locally

DuckDB can query the CSVs directly without any server:
```bash
# Install: https://duckdb.org/docs/installation
duckdb

D SELECT * FROM read_csv_auto('local-data/*.csv') LIMIT 10;
D SELECT uri_stem, COUNT(*) AS hits
  FROM read_csv_auto('local-data/top_uris.csv')
  GROUP BY uri_stem ORDER BY hits DESC;
```

---

## Stopping

```bash
docker compose down          # stop, keep grafana-data volume (dashboards/settings persist)
docker compose down -v       # stop + wipe all local state
```
