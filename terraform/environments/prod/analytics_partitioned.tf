# ── Partitioned Athena Table — Phase 7 (tracing/strategy.md) ─────────────────
#
# Requires the repartition Lambda (partition_lambda.tf) to have run at least once
# so that cf-logs-partitioned/year=.../month=.../day=.../ prefixes exist.
#
# Uses partition projection: no MSCK REPAIR, no Glue crawler.
# Athena infers partitions directly from the S3 path template.
# Saves ~95% scan cost vs querying the flat cf-logs/ prefix.
#
# Use this table (access_logs) instead of access_logs_raw for all new queries.
# Update views in docs/tracing/queries/views/ to reference cf_logs.access_logs.

resource "aws_glue_catalog_table" "access_logs" {
  name          = "access_logs"
  database_name = aws_glue_catalog_database.cf_logs.name
  table_type    = "EXTERNAL_TABLE"

  parameters = {
    "skip.header.line.count"              = "2"
    "compressionType"                     = "gzip"
    "classification"                      = "csv"
    "projection.enabled"                  = "true"
    "projection.year.type"                = "integer"
    "projection.year.range"               = "2026,2035"
    "projection.month.type"               = "integer"
    "projection.month.range"              = "1,12"
    "projection.month.digits"             = "2"
    "projection.day.type"                 = "integer"
    "projection.day.range"                = "1,31"
    "projection.day.digits"               = "2"
    "storage.location.template"           = "s3://${aws_s3_bucket.cf_logs.bucket}/cf-logs-partitioned/year=$${year}/month=$${month}/day=$${day}/"
  }

  partition_keys {
    name = "year"
    type = "int"
  }
  partition_keys {
    name = "month"
    type = "int"
  }
  partition_keys {
    name = "day"
    type = "int"
  }

  storage_descriptor {
    # Location is overridden per-partition via projection; this is a required placeholder.
    location      = "s3://${aws_s3_bucket.cf_logs.bucket}/cf-logs-partitioned/"
    input_format  = "org.apache.hadoop.mapred.TextInputFormat"
    output_format = "org.apache.hadoop.hive.ql.io.HiveIgnoreKeyTextOutputFormat"

    ser_de_info {
      serialization_library = "org.apache.hadoop.hive.serde2.lazy.LazySimpleSerDe"
      parameters = {
        "field.delim"          = "\t"
        "serialization.format" = "\t"
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
