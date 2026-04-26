-- Top 20 external referers last 7 days.
-- Filters out self-referrals and direct traffic ('-').
-- Workgroup: trellislab-analytics   Database: cf_logs

SELECT
  referer,
  COUNT(*) AS hits
FROM cf_logs.access_logs_raw
WHERE request_date >= current_date - interval '7' day
  AND referer NOT LIKE '%trellislab.net%'
  AND referer <> '-'
GROUP BY referer
ORDER BY hits DESC
LIMIT 20;
