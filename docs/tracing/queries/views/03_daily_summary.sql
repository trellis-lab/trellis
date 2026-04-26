-- VIEW: cf_logs.daily_summary
--
-- One row per day. Pre-aggregated metrics ready for Grafana panels.
-- This is the primary view wired to dashboard time-series charts.
--
-- Depends on: cf_logs.requests_enriched + cf_logs.sessions (run views 01+02 first).
-- Run once in Athena console (workgroup: trellislab-analytics, db: cf_logs).

CREATE OR REPLACE VIEW cf_logs.daily_summary AS
SELECT
  r.request_date,

  -- Volume
  COUNT(*)                                                        AS total_requests,
  COUNT(*) FILTER (WHERE r.browser_family <> 'bot')              AS human_requests,
  COUNT(*) FILTER (WHERE r.browser_family =  'bot')              AS bot_requests,

  -- Unique visitors (hashed IPs, humans only)
  COUNT(DISTINCT r.visitor_hash)
    FILTER (WHERE r.browser_family <> 'bot')                      AS unique_visitors,

  -- Sessions
  (
    SELECT COUNT(*)
    FROM cf_logs.sessions s
    WHERE s.request_date = r.request_date
  )                                                               AS sessions,

  -- Bandwidth
  SUM(r.bytes_sent)                                               AS bytes_total,
  SUM(r.bytes_sent) FILTER (WHERE r.uri_stem LIKE '/wasm/%')      AS bytes_wasm,

  -- Performance (humans only, successful responses)
  ROUND(AVG(r.time_taken)
    FILTER (WHERE r.browser_family <> 'bot' AND r.status < 400), 3) AS avg_response_sec,
  ROUND(
    APPROX_PERCENTILE(r.time_taken, 0.95)
    FILTER (WHERE r.browser_family <> 'bot' AND r.status < 400), 3) AS p95_response_sec,

  -- Status codes
  COUNT(*) FILTER (WHERE r.status BETWEEN 200 AND 299)            AS http_2xx,
  COUNT(*) FILTER (WHERE r.status BETWEEN 300 AND 399)            AS http_3xx,
  COUNT(*) FILTER (WHERE r.status BETWEEN 400 AND 499)            AS http_4xx,
  COUNT(*) FILTER (WHERE r.status >= 500)                         AS http_5xx,

  -- CloudFront cache
  COUNT(*) FILTER (WHERE r.edge_result_type = 'Hit')              AS cf_cache_hits,
  COUNT(*) FILTER (WHERE r.edge_result_type = 'Miss')             AS cf_cache_misses,
  ROUND(
    100.0 * COUNT(*) FILTER (WHERE r.edge_result_type = 'Hit')
    / NULLIF(COUNT(*), 0),
  1)                                                              AS cf_hit_rate_pct,

  -- Top country (by request count, humans only)
  ARBITRARY(r.country_name) FILTER (
    WHERE r.browser_family <> 'bot'
      AND r.country_name = (
        SELECT country_name
        FROM cf_logs.requests_enriched
        WHERE request_date = r.request_date
          AND browser_family <> 'bot'
        GROUP BY country_name
        ORDER BY COUNT(*) DESC
        LIMIT 1
      )
  )                                                               AS top_country

FROM cf_logs.requests_enriched r
GROUP BY r.request_date;
