# GitHub OIDC provider — allows GitHub Actions to assume AWS roles without stored keys
resource "aws_iam_openid_connect_provider" "github" {
  url             = "https://token.actions.githubusercontent.com"
  client_id_list  = ["sts.amazonaws.com"]
  thumbprint_list = ["6938fd4d98bab03faadb97b34396831e3780aea1"]
}

# IAM role assumed by the GitHub Actions prod deployment job
resource "aws_iam_role" "github_actions_prod" {
  name = "github-actions-trellislab-prod"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Principal = {
        Federated = aws_iam_openid_connect_provider.github.arn
      }
      Action = "sts:AssumeRoleWithWebIdentity"
      Condition = {
        StringEquals = {
          "token.actions.githubusercontent.com:aud" = "sts.amazonaws.com"
        }
        StringLike = {
          # Scoped to the prod GitHub Environment — dev/stage branches cannot assume this role
          "token.actions.githubusercontent.com:sub" = "repo:${var.github_repo}:environment:prod"
        }
      }
    }]
  })
}

resource "aws_iam_role_policy" "github_actions_prod" {
  name = "trellislab-prod-deploy"
  role = aws_iam_role.github_actions_prod.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "S3Deploy"
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "s3:ListBucket",
        ]
        Resource = [
          aws_s3_bucket.website.arn,
          "${aws_s3_bucket.website.arn}/*",
        ]
      },
      {
        Sid      = "CloudFrontInvalidate"
        Effect   = "Allow"
        Action   = "cloudfront:CreateInvalidation"
        Resource = aws_cloudfront_distribution.website.arn
      },
    ]
  })
}
