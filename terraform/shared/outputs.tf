output "zone_id" {
  value       = aws_route53_zone.main.zone_id
  description = "Route 53 hosted zone ID — used by prod environment"
}

output "name_servers" {
  value       = aws_route53_zone.main.name_servers
  description = "Update these 4 nameservers at GoDaddy before applying prod"
}

output "certificate_arn" {
  value       = aws_acm_certificate_validation.main.certificate_arn
  description = "Validated ACM cert ARN — used by CloudFront in prod"
}
