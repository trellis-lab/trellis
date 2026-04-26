-- VIEW: cf_logs.requests_enriched
--
-- Adds browser_family, device_class, country_code, country_name, region,
-- and hashed visitor_id to every raw log row.
--
-- Geo uses CloudFront edge PoP code (first 3 chars of edge_location).
-- This gives the serving edge location, not the exact viewer location —
-- accurate enough for country-level stats at Phase 3 scale.
-- Upgrade to MaxMind GeoLite2 IP range join for city-level accuracy.
--
-- Run once in Athena console (workgroup: trellislab-analytics, db: cf_logs).
-- Re-run to update the view definition after schema changes.

CREATE OR REPLACE VIEW cf_logs.requests_enriched AS
SELECT
  r.*,

  -- Browser family (order matters: Edge contains 'Chrome/', check it first)
  CASE
    WHEN regexp_like(r.user_agent, '(?i)bot|crawler|spider|slurp|bingbot|googlebot|curl|wget|python|java|libwww') THEN 'bot'
    WHEN regexp_like(r.user_agent, 'Edg/')        THEN 'Edge'
    WHEN regexp_like(r.user_agent, 'Chrome/')     THEN 'Chrome'
    WHEN regexp_like(r.user_agent, 'Firefox/')    THEN 'Firefox'
    WHEN regexp_like(r.user_agent, 'Safari/')     THEN 'Safari'
    WHEN regexp_like(r.user_agent, 'OPR/|Opera')  THEN 'Opera'
    ELSE 'other'
  END AS browser_family,

  -- Device class
  CASE
    WHEN regexp_like(r.user_agent, '(?i)Mobile|Android(?!.*Tablet)|iPhone|Windows Phone') THEN 'mobile'
    WHEN regexp_like(r.user_agent, '(?i)Tablet|iPad|Android.*Tablet')                     THEN 'tablet'
    ELSE 'desktop'
  END AS device_class,

  -- Geo from PoP code (coarse: edge location, not viewer IP)
  COALESCE(p.country_code, 'XX')   AS country_code,
  COALESCE(p.country_name, 'Unknown') AS country_name,
  COALESCE(p.region, 'Unknown')    AS region,

  -- Privacy-safe visitor fingerprint: SHA-256(ip + salt)
  -- Replace 'trellis-salt-2026' with a value kept outside the repo.
  to_hex(sha256(to_utf8(r.client_ip || 'trellis-salt-2026'))) AS visitor_hash,

  -- Parsed timestamp for time-based analysis
  date_parse(
    CAST(r.request_date AS varchar) || ' ' || r.request_time,
    '%Y-%m-%d %H:%i:%s'
  ) AS request_ts

FROM cf_logs.access_logs_raw r
LEFT JOIN cf_logs.pop_to_country p
  ON p.pop_code = UPPER(SUBSTR(r.edge_location, 1, 3));
