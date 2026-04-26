variable "domain" {
  default = "trellislab.net"
}


variable "github_repo" {
  default = "trellis-mermaid/trellis"
}

variable "aws_region" {
  default = "eu-central-1"
}

variable "enable_waf" {
  description = "Enable WAF WebACL on the CloudFront distribution"
  type        = bool
  default     = true
}

variable "waf_rate_limit" {
  description = "Max requests per IP per 5 minutes before blocking"
  type        = number
  default     = 2000
}

data "aws_caller_identity" "current" {}
