-- HTTP status code distribution for the last 7 days.
-- Use this to spot 4xx/5xx spikes.
-- Workgroup: trellislab-analytics   Database: cf_logs

SELECT
  request_date,
  status,
  COUNT(*) AS hits
FROM cf_logs.access_logs_raw
WHERE request_date >= current_date - interval '7' day
GROUP BY request_date, status
ORDER BY request_date, status;
