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
