# WAF WebACL — attached to CloudFront; must be created in us-east-1 (CLOUDFRONT scope)
resource "aws_wafv2_web_acl" "website" {
  count       = var.enable_waf ? 1 : 0
  provider    = aws.us_east_1
  name        = "trellislab-prod-waf"
  description = "WAF for trellislab.net CloudFront distribution"
  scope       = "CLOUDFRONT"

  default_action {
    allow {}
  }

  # Priority 1: Block IPs on AWS threat intelligence reputation list
  rule {
    name     = "AWSManagedRulesAmazonIpReputationList"
    priority = 1

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesAmazonIpReputationList"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "IpReputationList"
      sampled_requests_enabled   = true
    }
  }

  # Priority 2: Block common scanner payloads (log4j, SSRF, path traversal, etc.)
  rule {
    name     = "AWSManagedRulesKnownBadInputsRuleSet"
    priority = 2

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesKnownBadInputsRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "KnownBadInputs"
      sampled_requests_enabled   = true
    }
  }

  # Priority 3: Rate limit — block IPs exceeding 2000 requests per 5 minutes
  rule {
    name     = "RateLimitPerIP"
    priority = 3

    action {
      block {}
    }

    statement {
      rate_based_statement {
        limit              = var.waf_rate_limit
        aggregate_key_type = "IP"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "RateLimitPerIP"
      sampled_requests_enabled   = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "trellislab-prod-waf"
    sampled_requests_enabled   = true
  }

  tags = {
    Project     = "trellislab"
    Environment = "prod"
  }
}
