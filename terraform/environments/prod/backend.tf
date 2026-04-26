terraform {
  required_version = ">= 1.6"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {
    bucket         = "trellislab-terraform-state"
    key            = "environments/prod/terraform.tfstate"
    region         = "eu-central-1"
    dynamodb_table = "terraform-state-locks"
    encrypt        = true
  }
}

provider "aws" {
  region = "eu-central-1"
}

# WAF for CloudFront must be managed in us-east-1 (CLOUDFRONT scope requirement)
provider "aws" {
  alias  = "us_east_1"
  region = "us-east-1"
}
