# ── Athena Analytics — Phase 2 (tracing/strategy.md) ──────────────────────────

# S3 bucket for Athena query results (30-day auto-delete)
resource "aws_s3_bucket" "athena_results" {
  bucket = "trellislab-athena-results"
}

resource "aws_s3_bucket_server_side_encryption_configuration" "athena_results" {
  bucket = aws_s3_bucket.athena_results.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "athena_results" {
  bucket                  = aws_s3_bucket.athena_results.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_lifecycle_configuration" "athena_results" {
  bucket = aws_s3_bucket.athena_results.id
  rule {
    id     = "expire-query-results"
    status = "Enabled"
    expiration {
      days = 30
    }
  }
}

# Athena workgroup — separate cost tracking, enforces 100 MB per-query scan cap
resource "aws_athena_workgroup" "analytics" {
  name = "trellislab-analytics"

  configuration {
    result_configuration {
      output_location = "s3://${aws_s3_bucket.athena_results.bucket}/results/"
    }
    bytes_scanned_cutoff_per_query     = 104857600 # 100 MB hard cap per query
    publish_cloudwatch_metrics_enabled = false
  }
}

# Glue database for cf_logs schema
resource "aws_glue_catalog_database" "cf_logs" {
  name = "cf_logs"
}

# External table over raw CloudFront standard log files (.gz TSV, 2-line header)
resource "aws_glue_catalog_table" "access_logs_raw" {
  name          = "access_logs_raw"
  database_name = aws_glue_catalog_database.cf_logs.name
  table_type    = "EXTERNAL_TABLE"

  parameters = {
    "skip.header.line.count" = "2"
    "compressionType"        = "gzip"
    "classification"         = "csv"
  }

  storage_descriptor {
    location      = "s3://${aws_s3_bucket.cf_logs.bucket}/cf-logs/"
    input_format  = "org.apache.hadoop.mapred.TextInputFormat"
    output_format = "org.apache.hadoop.hive.ql.io.HiveIgnoreKeyTextOutputFormat"

    ser_de_info {
      serialization_library = "org.apache.hadoop.hive.serde2.lazy.LazySimpleSerDe"
      parameters = {
        "field.delim"            = "\t"
        "serialization.format"   = "\t"
      }
    }

    columns {
      name = "request_date"
      type = "date"
    }
    columns {
      name = "request_time"
      type = "string"
    }
    columns {
      name = "edge_location"
      type = "string"
    }
    columns {
      name = "bytes_sent"
      type = "bigint"
    }
    columns {
      name = "client_ip"
      type = "string"
    }
    columns {
      name = "http_method"
      type = "string"
    }
    columns {
      name = "cs_host"
      type = "string"
    }
    columns {
      name = "uri_stem"
      type = "string"
    }
    columns {
      name = "status"
      type = "int"
    }
    columns {
      name = "referer"
      type = "string"
    }
    columns {
      name = "user_agent"
      type = "string"
    }
    columns {
      name = "uri_query"
      type = "string"
    }
    columns {
      name = "cookie"
      type = "string"
    }
    columns {
      name = "edge_result_type"
      type = "string"
    }
    columns {
      name = "edge_request_id"
      type = "string"
    }
    columns {
      name = "host_header"
      type = "string"
    }
    columns {
      name = "protocol"
      type = "string"
    }
    columns {
      name = "cs_bytes"
      type = "bigint"
    }
    columns {
      name = "time_taken"
      type = "double"
    }
    columns {
      name = "forwarded_for"
      type = "string"
    }
    columns {
      name = "ssl_protocol"
      type = "string"
    }
    columns {
      name = "ssl_cipher"
      type = "string"
    }
    columns {
      name = "edge_response_result_type"
      type = "string"
    }
    columns {
      name = "protocol_version"
      type = "string"
    }
    columns {
      name = "fle_status"
      type = "string"
    }
    columns {
      name = "fle_encrypted_fields"
      type = "int"
    }
    columns {
      name = "c_port"
      type = "int"
    }
    columns {
      name = "time_to_first_byte"
      type = "double"
    }
    columns {
      name = "edge_detailed_result_type"
      type = "string"
    }
    columns {
      name = "sc_content_type"
      type = "string"
    }
    columns {
      name = "sc_content_len"
      type = "bigint"
    }
    columns {
      name = "sc_range_start"
      type = "bigint"
    }
    columns {
      name = "sc_range_end"
      type = "bigint"
    }
  }
}
