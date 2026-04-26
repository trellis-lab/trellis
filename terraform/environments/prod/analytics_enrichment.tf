# ── Athena Enrichment Tables — Phase 3 (tracing/strategy.md) ─────────────────
#
# The pop_to_country.csv reference data must be uploaded to S3 once before
# the Athena table can be queried:
#
#   aws s3 cp docs/tracing/data/pop_to_country.csv \
#     s3://trellislab-cloudfront-logs/ref-data/pop_to_country.csv \
#     --profile trellislab
#
# After that, the Glue table below makes it queryable from Athena.

resource "aws_glue_catalog_table" "pop_to_country" {
  name          = "pop_to_country"
  database_name = aws_glue_catalog_database.cf_logs.name
  table_type    = "EXTERNAL_TABLE"

  parameters = {
    "skip.header.line.count" = "1"
    "classification"         = "csv"
  }

  storage_descriptor {
    location      = "s3://${aws_s3_bucket.cf_logs.bucket}/ref-data/"
    input_format  = "org.apache.hadoop.mapred.TextInputFormat"
    output_format = "org.apache.hadoop.hive.ql.io.HiveIgnoreKeyTextOutputFormat"

    ser_de_info {
      serialization_library = "org.apache.hadoop.hive.serde2.lazy.LazySimpleSerDe"
      parameters = {
        "field.delim"          = ","
        "serialization.format" = ","
      }
    }

    columns {
      name = "pop_code"
      type = "string"
    }
    columns {
      name = "country_code"
      type = "string"
    }
    columns {
      name = "country_name"
      type = "string"
    }
    columns {
      name = "region"
      type = "string"
    }
  }
}
