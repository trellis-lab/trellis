output "website_url" {
  value = "https://${var.domain}"
}

output "s3_bucket_name" {
  value       = aws_s3_bucket.website.bucket
  description = "S3 bucket receiving deployments"
}

output "cloudfront_distribution_id" {
  value       = aws_cloudfront_distribution.website.id
  description = "Set as CLOUDFRONT_DISTRIBUTION_ID variable in GitHub prod environment"
}

output "cloudfront_domain_name" {
  value = aws_cloudfront_distribution.website.domain_name
}

output "github_actions_role_arn" {
  value       = aws_iam_role.github_actions_prod.arn
  description = "Set as IAM_ROLE_ARN variable in GitHub prod environment"
}

output "cf_logs_bucket" {
  value       = aws_s3_bucket.cf_logs.bucket
  description = "S3 bucket receiving CloudFront access logs"
}

output "athena_workgroup" {
  value       = aws_athena_workgroup.analytics.name
  description = "Athena workgroup for analytics queries"
}

output "athena_results_bucket" {
  value       = aws_s3_bucket.athena_results.bucket
  description = "S3 bucket storing Athena query results (30-day TTL)"
}
