# Deployment Strategy — trellislab.net

**Date:** 2026-04-24  
**Scope:** Phase 0 — production website, first live iteration  
**GitHub repo:** `github.com/trellis-mermaid/trellis`

---

## 1. Situation

| Item | Status |
|------|--------|
| Domain `trellislab.net` | Registered at GoDaddy |
| ProtonMail workspace | Live; MX/TXT/SPF/DKIM/DMARC already set in GoDaddy DNS |
| Web content (`/web`) | `index.html`, `editor.html`, `favicon.ico`, `assets/`, `wasm/` |
| WASM local build | Present but must be rebuilt from source on deploy |

---

## 2. Phase 0 Scope

Keep it minimal. Ship a working production site first.

**In scope:**
- Single production environment (`trellislab.net`, `www.trellislab.net`)
- S3 (private) + CloudFront + ACM wildcard cert
- Route 53 as authoritative DNS (replaces GoDaddy DNS)
- DNS migration that preserves existing ProtonMail records
- CI/CD: GitHub Actions — build WASM, sync to S3, invalidate CloudFront
- GitHub OIDC for AWS auth (no stored secrets)
- GA4 analytics (script tag only, no AWS infrastructure)
- Terraform IaC for all AWS resources

**Out of scope (phase 1+):**
- dev / stage environments
- WAF v2
- Lambda + API Gateway + DynamoDB (no dynamic backend needed)
- Geo-redundancy / multi-region

---

## 3. Architecture (Phase 0)

```
GoDaddy (registrar, nameservers only)
    │ NS records → Route 53
    ▼
Route 53 Hosted Zone (trellislab.net)
    │ A/AAAA Alias → CloudFront
    │ MX/TXT/CNAME → ProtonMail (migrated from GoDaddy)
    ▼
CloudFront Distribution
    │ TLS via ACM wildcard (us-east-1)
    │ CloudFront Function: URL rewrite (dir → /index.html)
    │ Response Headers Policy: HSTS, X-Frame-Options, etc.
    │ Compression: gzip + brotli
    ▼
S3 Bucket (private, OAC only)
    HTML  — max-age=300
    WASM  — max-age=31536000, immutable
    Assets — max-age=86400
```

---

## 4. DNS Migration Plan

**Critical:** ProtonMail email must not break. Migration sequence:

1. Create Route 53 hosted zone → note the 4 nameserver addresses
2. In Route 53, recreate all current GoDaddy DNS records:
   - `A/AAAA` → CloudFront alias (new)
   - `MX` → `mail.protonmail.ch` (copy from GoDaddy)
   - `TXT` → SPF + ProtonMail verification (copy from GoDaddy)
   - `CNAME protonmail._domainkey.*` × 3 → DKIM (copy from GoDaddy)
   - `TXT _dmarc.*` → DMARC policy (copy from GoDaddy)
3. Verify all ProtonMail records are correct in Route 53 before switching
4. At GoDaddy: change nameservers to Route 53 NS values
5. Wait for propagation (~24–48h), verify email still works
6. Remove old DNS records from GoDaddy (nameservers now authoritative via Route 53)

---

## 5. Terraform Structure

```
terraform/
├── shared/                  # Apply once
│   ├── main.tf              # Route 53 zone, ACM cert
│   └── email.tf             # ProtonMail MX/SPF/DKIM/DMARC records
│
└── environments/
    └── prod/
        └── main.tf          # S3, CloudFront, OAC, IAM, GitHub OIDC role
```

S3 remote state backend + DynamoDB lock table created manually before first `terraform init`.

---

## 6. CI/CD Pipeline

Single workflow: `.github/workflows/deploy.yml`

```
Trigger: push to main (or manual dispatch)

Steps:
1. checkout
2. Install Rust + wasm32-unknown-unknown target + wasm-pack
3. wasm-pack build trellis-wasm --target web --release
4. Assemble dist/: cp web/* dist/, cp trellis-wasm/pkg/* dist/wasm/
5. Configure AWS credentials via OIDC (assume prod IAM role)
6. aws s3 sync dist/ s3://trellislab-prod --delete
   --content-type overrides for .wasm → application/wasm
   --cache-control per file type
7. aws cloudfront create-invalidation --paths "/*"
```

IAM role trust policy scoped to: `repo:trellis-mermaid/trellis:environment:prod`

---

## 7. Implementation Sequence

ACM cert validation requires Route 53 to be authoritative, which requires GoDaddy NS switch first. Apply shared in two passes to avoid a 75-minute timeout.

| Step | Command / Action | Notes |
|------|-----------------|-------|
| 1 | `cd terraform/bootstrap && terraform init && terraform apply` | Creates S3 state bucket + DynamoDB lock |
| 2 | `cd terraform/shared && terraform init` | Initialises remote state backend |
| 3 | `terraform apply -target=aws_route53_zone.main` | Creates hosted zone only |
| 4 | `terraform output name_servers` | Copy the 4 NS addresses |
| 5 | At GoDaddy: update nameservers to the 4 Route 53 NS values | DNS propagation: 1–48h |
| 6 | Verify with `nslookup -type=NS trellislab.net 8.8.8.8` until Route 53 NS appear | |
| 7 | `terraform apply` (full shared apply) | Creates ACM cert + ProtonMail records; cert validates in ~10 min |
| 8 | `cd terraform/environments/prod && terraform init && terraform apply` | Creates S3, CloudFront, IAM OIDC role |
| 9 | `terraform output` — note `cloudfront_distribution_id` and `github_actions_role_arn` | |
| 10 | GitHub → repo → Settings → Environments → New environment: `prod` | |
| 11 | Add environment variables: `CLOUDFRONT_DISTRIBUTION_ID`, `IAM_ROLE_ARN` | Values from step 9 |
| 12 | Push to `main` → first automated deploy | CI builds WASM + syncs to S3 |
| 13 | Smoke test: `https://trellislab.net` loads, WASM works, email still delivers | |

---

## 8. Resolved Questions

| # | Question | Answer |
|---|----------|--------|
| 1 | GoDaddy nameserver access | ✅ Yes |
| 2 | Export current DNS records | ✅ Yes |
| 3 | AWS account exists | ✅ Yes (see §9 for what's needed) |
| 4 | GitHub repo | `github.com/trellis-mermaid/trellis` (may rename later) |
| 5 | Dynamic backend needed | ✅ No — static HTML + WASM only |
| 6 | Current DMARC policy | `p=quarantine; adkim=r; aspf=r; rua=mailto:dmarc_rua@onsecureserver.net` |

---

## 9. What Is Needed to Start

To generate and apply the Terraform, provide:

| Item | Why needed |
|------|-----------|
| **AWS account ID** (12-digit) | ARN references in IAM + OIDC trust policy |
| **Preferred AWS region** | All resources except ACM cert (which is always `us-east-1`) |
| **Current GoDaddy DNS records** | Recreate ProtonMail records in Route 53 before switching NS |

Everything else (S3, CloudFront, Route 53, GitHub Actions workflow) can be written and applied with those three inputs.
