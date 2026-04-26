-- Top 20 requested URIs yesterday.
-- Change the date filter to query a different day.
-- Workgroup: trellislab-analytics   Database: cf_logs

SELECT
  uri_stem,
  COUNT(*)                        AS hits,
  SUM(bytes_sent)                 AS bytes_total,
  ROUND(AVG(time_taken), 3)       AS avg_time_sec
FROM cf_logs.access_logs_raw
WHERE request_date = current_date - interval '1' day
GROUP BY uri_stem
ORDER BY hits DESC
LIMIT 20;
