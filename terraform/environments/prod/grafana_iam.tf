# ── Grafana Read-Only IAM User — Phase 4 (tracing/strategy.md) ───────────────
#
# Used by the local Grafana instance to query Athena.
# After apply, generate an access key manually:
#   AWS Console → IAM → Users → grafana-readonly → Security credentials
#   → Create access key → Application running outside AWS
# Then copy to docs/tracing/grafana/.env (gitignored).

resource "aws_iam_user" "grafana_readonly" {
  name = "grafana-readonly"
}

resource "aws_iam_user_policy" "grafana_readonly" {
  name = "grafana-readonly-policy"
  user = aws_iam_user.grafana_readonly.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AthenaQueryExecution"
        Effect = "Allow"
        Action = [
          "athena:StartQueryExecution",
          "athena:StopQueryExecution",
          "athena:GetQueryExecution",
          "athena:GetQueryResults",
          "athena:GetQueryResultsStream",
          "athena:ListQueryExecutions",
          "athena:GetWorkGroup",
        ]
        Resource = aws_athena_workgroup.analytics.arn
      },
      {
        Sid    = "AthenaListWorkgroups"
        Effect = "Allow"
        Action = ["athena:ListWorkGroups"]
        Resource = "*"
      },
      {
        Sid    = "S3ReadLogs"
        Effect = "Allow"
        Action = ["s3:GetObject", "s3:ListBucket"]
        Resource = [
          aws_s3_bucket.cf_logs.arn,
          "${aws_s3_bucket.cf_logs.arn}/*",
        ]
      },
      {
        Sid    = "S3ReadWriteAthenaResults"
        Effect = "Allow"
        Action = ["s3:GetObject", "s3:PutObject", "s3:ListBucket", "s3:GetBucketLocation"]
        Resource = [
          aws_s3_bucket.athena_results.arn,
          "${aws_s3_bucket.athena_results.arn}/*",
        ]
      },
      {
        Sid    = "GlueReadCfLogs"
        Effect = "Allow"
        Action = [
          "glue:GetDatabase",
          "glue:GetDatabases",
          "glue:GetTable",
          "glue:GetTables",
          "glue:GetPartition",
          "glue:GetPartitions",
          "glue:BatchGetPartition",
        ]
        Resource = [
          "arn:aws:glue:${var.aws_region}:${data.aws_caller_identity.current.account_id}:catalog",
          "arn:aws:glue:${var.aws_region}:${data.aws_caller_identity.current.account_id}:database/cf_logs",
          "arn:aws:glue:${var.aws_region}:${data.aws_caller_identity.current.account_id}:table/cf_logs/*",
        ]
      }
    ]
  })
}

output "grafana_iam_user" {
  value       = aws_iam_user.grafana_readonly.name
  description = "Create an access key for this user and add to docs/tracing/grafana/.env"
}
