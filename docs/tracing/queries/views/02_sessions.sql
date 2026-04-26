-- VIEW: cf_logs.sessions
--
-- One row per (visitor_hash, request_date) session.
-- Session boundary: same hashed visitor within a calendar day.
-- Excludes bots. Use this for daily unique visitor counts.
--
-- Limitation: a single visitor spanning midnight counts as two sessions.
-- Accurate enough for early-stage analytics; upgrade to LAG()-based
-- 30-min gap detection when you need precise session duration.
--
-- Depends on: cf_logs.requests_enriched (run view 01 first).
-- Run once in Athena console (workgroup: trellislab-analytics, db: cf_logs).

CREATE OR REPLACE VIEW cf_logs.sessions AS
SELECT
  visitor_hash,
  request_date,
  country_code,
  country_name,
  region,
  browser_family,
  device_class,
  COUNT(*)                                                        AS hits,
  COUNT(DISTINCT uri_stem)                                        AS unique_pages,
  MIN(request_ts)                                                 AS session_start,
  MAX(request_ts)                                                 AS session_end,
  DATE_DIFF(
    'minute',
    MIN(request_ts),
    MAX(request_ts)
  )                                                               AS duration_minutes,
  ARBITRARY(referer) FILTER (WHERE referer <> '-')                AS entry_referer
FROM cf_logs.requests_enriched
WHERE browser_family <> 'bot'
GROUP BY
  visitor_hash,
  request_date,
  country_code,
  country_name,
  region,
  browser_family,
  device_class;
