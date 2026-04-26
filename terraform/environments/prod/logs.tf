# ── CloudFront Access Logs — Phase 1 (tracing/strategy.md) ────────────────────

resource "aws_s3_bucket" "cf_logs" {
  bucket = "trellislab-cloudfront-logs"
}

# CloudFront standard log delivery requires BucketOwnerPreferred + log-delivery-write ACL
resource "aws_s3_bucket_ownership_controls" "cf_logs" {
  bucket = aws_s3_bucket.cf_logs.id
  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_acl" "cf_logs" {
  depends_on = [aws_s3_bucket_ownership_controls.cf_logs]
  bucket     = aws_s3_bucket.cf_logs.id
  acl        = "log-delivery-write"
}

resource "aws_s3_bucket_server_side_encryption_configuration" "cf_logs" {
  bucket = aws_s3_bucket.cf_logs.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "cf_logs" {
  bucket                  = aws_s3_bucket.cf_logs.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# ── Phase 7: S3 lifecycle — cost hardening ─────────────────────────────────────
resource "aws_s3_bucket_lifecycle_configuration" "cf_logs" {
  bucket = aws_s3_bucket.cf_logs.id

  # Raw flat logs (written by CloudFront)
  rule {
    id     = "raw-logs-tiering"
    status = "Enabled"
    filter { prefix = "cf-logs/" }
    transition {
      days          = 30
      storage_class = "STANDARD_IA"
    }
    transition {
      days          = 90
      storage_class = "GLACIER_IR"
    }
    expiration {
      days = 365
    }
  }

  # Partitioned copies (written by the repartition Lambda)
  # Kept longer in IA because Athena queries hit this prefix.
  rule {
    id     = "partitioned-logs-tiering"
    status = "Enabled"
    filter { prefix = "cf-logs-partitioned/" }
    transition {
      days          = 60
      storage_class = "STANDARD_IA"
    }
    transition {
      days          = 180
      storage_class = "GLACIER_IR"
    }
    expiration {
      days = 365
    }
  }

}
