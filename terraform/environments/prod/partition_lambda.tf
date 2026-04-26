# ── Log Repartitioner Lambda — Phase 7 (tracing/strategy.md) ─────────────────
#
# Runs daily at 02:00 UTC. Copies flat cf-logs/<file>.gz into
# cf-logs-partitioned/year=YYYY/month=MM/day=DD/ so Athena partition
# projection can skip unrelated dates (saves ~95% scan cost).

data "archive_file" "partition_logs" {
  type        = "zip"
  source_file = "${path.module}/lambda/partition_logs.py"
  output_path = "${path.module}/lambda/partition_logs.zip"
}

resource "aws_iam_role" "partition_logs_lambda" {
  name = "trellislab-partition-logs-lambda"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "lambda.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy" "partition_logs_lambda" {
  name = "trellislab-partition-logs-policy"
  role = aws_iam_role.partition_logs_lambda.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "ReadWriteLogsBucket"
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:ListBucket",
        ]
        Resource = [
          aws_s3_bucket.cf_logs.arn,
          "${aws_s3_bucket.cf_logs.arn}/*",
        ]
      },
      {
        Sid      = "WriteLogs"
        Effect   = "Allow"
        Action   = ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"]
        Resource = "arn:aws:logs:*:*:*"
      }
    ]
  })
}

resource "aws_lambda_function" "partition_logs" {
  function_name    = "trellislab-partition-logs"
  role             = aws_iam_role.partition_logs_lambda.arn
  runtime          = "python3.12"
  handler          = "partition_logs.lambda_handler"
  filename         = data.archive_file.partition_logs.output_path
  source_code_hash = data.archive_file.partition_logs.output_base64sha256
  timeout          = 300 # 5 min; enough for a full day of logs at any expected volume

  environment {
    variables = {
      LOG_BUCKET = aws_s3_bucket.cf_logs.bucket
    }
  }
}

resource "aws_cloudwatch_log_group" "partition_logs_lambda" {
  name              = "/aws/lambda/${aws_lambda_function.partition_logs.function_name}"
  retention_in_days = 30
}

# EventBridge Scheduler: daily at 02:00 UTC (after CloudFront finishes writing previous day)
resource "aws_scheduler_schedule" "partition_logs_daily" {
  name       = "trellislab-partition-logs-daily"
  group_name = "default"

  flexible_time_window { mode = "OFF" }
  schedule_expression          = "cron(0 2 * * ? *)"
  schedule_expression_timezone = "UTC"

  target {
    arn      = aws_lambda_function.partition_logs.arn
    role_arn = aws_iam_role.scheduler_partition_logs.arn
    input    = "{}"
  }
}

resource "aws_iam_role" "scheduler_partition_logs" {
  name = "trellislab-scheduler-partition-logs"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "scheduler.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy" "scheduler_partition_logs" {
  name = "trellislab-scheduler-partition-logs-invoke"
  role = aws_iam_role.scheduler_partition_logs.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = "lambda:InvokeFunction"
      Resource = aws_lambda_function.partition_logs.arn
    }]
  })
}
