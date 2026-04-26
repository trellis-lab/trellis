-- CloudFront cache hit/miss rate for WASM assets, last 7 days.
-- Hit = edge served from cache (Miss = fetched from S3 origin).
-- edge_result_type values: Hit, Miss, RefreshHit, Error, LimitExceeded
-- Workgroup: trellislab-analytics   Database: cf_logs

SELECT
  request_date,
  edge_result_type,
  COUNT(*)                                              AS requests,
  ROUND(
    100.0 * COUNT(*) / SUM(COUNT(*)) OVER (PARTITION BY request_date),
    1
  )                                                     AS pct
FROM cf_logs.access_logs_raw
WHERE request_date >= current_date - interval '7' day
  AND uri_stem LIKE '/wasm/%'
GROUP BY request_date, edge_result_type
ORDER BY request_date, edge_result_type;
