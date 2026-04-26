-- Daily request count for the last 7 days.
-- Excludes favicon, robots.txt, and health-check paths.
-- Workgroup: trellislab-analytics   Database: cf_logs

SELECT
  request_date,
  COUNT(*)                                          AS total_hits,
  COUNT(*) FILTER (WHERE status < 400)              AS successful_hits,
  COUNT(*) FILTER (WHERE status >= 400)             AS error_hits
FROM cf_logs.access_logs_raw
WHERE request_date >= current_date - interval '7' day
  AND uri_stem NOT IN ('/favicon.ico', '/robots.txt')
GROUP BY request_date
ORDER BY request_date;
