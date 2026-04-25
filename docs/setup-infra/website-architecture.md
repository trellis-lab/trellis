# AWS Static Website Architecture

## Technical Architecture Document

**Version:** 1.0 **Date:** April 22, 2026 **Status:** Final




## 1. Executive Summary

This document describes the software architecture for a static website platform hosted on Amazon Web Services (AWS). The platform serves a Rust/WebAssembly product alongside traditional HTML/CSS/JavaScript pages, with a serverless backend for dynamic functionality. The architecture supports three deployment environments (dev, stage, prod), custom domain with ProtonMail email integration, global content delivery, and a fully automated CI/CD pipeline.

The system is designed for low operational overhead, near-zero idle cost, and the ability to scale globally without infrastructure changes.




## 2. Requirements

### 2.1 Functional Requirements

- Host up to 5 static web pages (HTML/CSS/JavaScript)
- Embed and serve a Rust/WebAssembly product via wasm-pack
- Provide backend API functionality for dynamic operations (form submissions, data processing)
- Support a custom `.dev` domain with HTTPS
- Integrate ProtonMail for company email on the same domain
- Collect webpage analytics for marketing and product enhancement
- Support three deployment stages: development, staging, production

### 2.2 Non-Functional Requirements

- Global content delivery with low-latency access from Europe and Asia
- Automatic SSL certificate management (no manual renewals)
- Infrastructure as Code (Terraform) for repeatable, auditable deployments
- Automated CI/CD with branch-based environment promotion
- Security hardening (WAF, security headers, least-privilege IAM)
- Cost efficiency at low traffic volumes (target: under $10/month for prod)




## 3. High-Level Architecture

The architecture follows a serverless, edge-first pattern. All static content (including WASM binaries) is served through CloudFront's global CDN from a private S3 bucket. Dynamic API requests are routed through the same CloudFront distribution to a Lambda-backed API Gateway, keeping everything under a single domain with no CORS complexity.

```javascript
┌─────────────────────────────────────────────────────────────────┐
│                         END USERS                               │
│                    (Browser / Mobile)                           │
└──────────────────────────┬──────────────────────────────────────┘
                           │ HTTPS
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                     ROUTE 53 (DNS)                              │
│         yourdomain.dev → CloudFront Alias                       │
│         MX/SPF/DKIM/DMARC → ProtonMail                          │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                  CLOUDFRONT (CDN + TLS)                         │
│     ┌──────────────┐              ┌─────────────────┐           │
│     │ /* (default) │              │ /api/* (backend)│           │
│     │ → S3 Origin  │              │ → API Gateway   │           │
│     └──────┬───────┘              └───────┬─────────┘           │
│            │                              │                     │
│   URL Rewrite Function          WAF v2 (rate limit,             │
│   Security Headers              common rules, bad inputs)       │
│   Gzip/Brotli Compression                                       │
│   HTTP/2 + HTTP/3                                               │
└────────┬──────────────────────────────────┬─────────────────────┘
         │                                  │
         ▼                                  ▼
┌─────────────────┐              ┌─────────────────────────┐
│   S3 BUCKET     │              │   API GATEWAY (HTTP).   │
│   (Private)     │              │   ANY /api/v1/{proxy+}  │
│                 │              └──────────┬──────────────┘
│  HTML/CSS/JS    │                         │
│  WASM binaries  │                         ▼
│  Static assets  │              ┌──────────────────────┐
│                 │              │   LAMBDA FUNCTION    │
│  OAC access     │              │   Node.js 24.x       │
│  Versioning     │              └──────────┬───────────┘
│  Encryption     │                         │
└─────────────────┘                         ▼
                                 ┌───────────────────────┐
                                 │   DYNAMODB TABLE      │
                                 │   Pay-per-request     │
                                 │   Single-table design │
                                 │   TTL + PITR          │
                                 └───────────────────────┘
```




## 4. Component Architecture

### 4.1 DNS Layer — Route 53

Route 53 serves as the authoritative DNS for the domain. The domain is registered at an external registrar (Route 53 does not support `.dev` TLDs) with nameservers pointing to a Route 53 hosted zone.

**Records managed:**

| Record Type | Name | Target | Purpose |
| --- | --- | --- | --- |
| A (Alias) | trellislab.net | CloudFront | Website (IPv4) |
| AAAA (Alias) | trellislab.net | CloudFront | Website (IPv6) |
| A (Alias) | [www.](http://www.yourdomain.dev/)trellislab.net | CloudFront | WWW redirect |
| MX | yourdomain.dev | mail.protonmail.ch | Email routing |
| TXT | yourdomain.dev | SPF + verification | Email authentication |
| CNAME | protonmail.\_domainkey.\* | ProtonMail | DKIM signing (x3) |
| TXT | \_dmarc.yourdomain.dev | DMARC policy | Email spoofing protection |
| CNAME | ACM validation | ACM | SSL certificate validation |

Alias queries to CloudFront incur no DNS query charges.

### 4.2 SSL/TLS — AWS Certificate Manager (ACM)

A single wildcard certificate covers all environments and subdomains:

- **Primary domain:** `yourdomain.dev`
- **SAN wildcard:** `*.yourdomain.dev`
- **Validation:** DNS (automated via Route 53 CNAME records)
- **Region:** `us-east-1` (required for CloudFront)
- **Renewal:** Fully automatic, \~60 days before expiration
- **Cost:** Free

### 4.3 Content Delivery — CloudFront

CloudFront serves as the unified entry point for all traffic. It provides TLS termination, edge caching, compression, and routing.

**Distribution configuration:**

| Setting | Value |
| --- | --- |
| HTTP version | HTTP/2 + HTTP/3 |
| Price class | PriceClass\_100 (dev/stage), PriceClass\_200 (prod) |
| Default root object | index.html |
| Viewer protocol | Redirect HTTP → HTTPS |
| Compression | Enabled (gzip + brotli) |

**Origin behaviors:**

| Path Pattern | Origin | Cache Policy | Notes |
| --- | --- | --- | --- |
| `/*` (default) | S3 via OAC | CachingOptimized | Static + WASM |
| `/api/*` | API Gateway | CachingDisabled | Dynamic backend |

**Edge functions:**

- **URL Rewrite (CloudFront Function):** Appends `/index.html` to directory-style URIs. Runs at viewer-request phase.
- **Security Headers (Response Headers Policy):** HSTS, X-Content-Type-Options, X-Frame-Options, Referrer-Policy, XSS-Protection, Permissions-Policy.

### 4.4 Static Content — S3

The S3 bucket stores all website files including HTML, CSS, JavaScript, and WASM binaries produced by `wasm-pack`. The bucket is fully private — no public access — with CloudFront Origin Access Control (OAC) as the sole reader.

**Bucket configuration:**

| Setting | Dev | Stage | Prod |
| --- | --- | --- | --- |
| Versioning | Disabled | Enabled | Enabled |
| Encryption | AES-256 | AES-256 | AES-256 |
| Public access | Blocked | Blocked | Blocked |
| CORS | Enabled (own domain) | Enabled | Enabled |

**WASM-specific considerations:**

- `.wasm` files are uploaded with explicit `Content-Type: application/wasm` to ensure `WebAssembly.instantiateStreaming()` works correctly
- WASM binaries receive long cache headers (`max-age=31536000, immutable`) because wasm-pack generates content-addressed filenames
- HTML files receive short cache headers (`max-age=60` in dev, `max-age=300` in prod) for fast iteration

### 4.5 Backend — API Gateway + Lambda

An HTTP API (API Gateway v2) routes all `/api/*` requests to a single Lambda function. HTTP API was chosen over REST API for lower cost and lower latency at this scale.

**Lambda configuration:**

| Setting | Dev | Stage | Prod |
| --- | --- | --- | --- |
| Runtime | Node.js 24.x | Node.js 24.x | Node.js 24.x |
| Memory | 128 MB | 128 MB | 256 MB |
| Timeout | 10 sec | 10 sec | 10 sec |
| Log retention | 7 days | 30 days | 90 days |

The Lambda function has scoped IAM permissions limited to its environment's DynamoDB table.

### 4.6 Data Store — DynamoDB

A single-table design with composite primary key (PK/SK) and a GSI for reverse lookups.

| Setting | Value |
| --- | --- |
| Billing mode | Pay-per-request (on-demand) |
| Keys | PK (String), SK (String) |
| GSI | GSI1PK / GSI1SK |
| TTL | Enabled (ExpiresAt attribute) |
| PITR | Enabled (prod only) |

### 4.7 Security — WAF v2

WAF is attached to the CloudFront distribution in production only (togglable via `enable_waf` variable).

**Active rules:**

| Priority | Rule | Action | Purpose |
| --- | --- | --- | --- |
| 1 | Rate limiting (2000 req/5min/IP) | Block | DDoS / abuse mitigation |
| 2 | AWSManagedRulesCommonRuleSet | Block\* | XSS, SQLi, bad bots |
| 3 | AWSManagedRulesKnownBadInputsRuleSet | Block | Log4j, SSRF, etc. |
| 4 | AWSManagedRulesBotControlRuleSet | (disabled) | Optional bot control |

\*SizeRestrictions\_BODY rule set to count-only mode to avoid blocking WASM-related payloads.

WAF logs are sent to CloudWatch with cookie headers redacted.

### 4.8 Email — ProtonMail Integration

Email and web hosting coexist on the same domain using separate DNS record types. MX records route email to ProtonMail servers while A/AAAA records route web traffic to CloudFront. There is no conflict between these record types.

**DMARC policy** starts at `p=none` (monitor only) and should be tightened to `p=quarantine` or `p=reject` after initial verification.

### 4.9 Analytics

Google Analytics 4 is recommended for the initial setup — it requires only a `<script>` tag in the HTML, no AWS infrastructure. For a future AWS-native pipeline, the architecture supports Pinpoint → Kinesis Firehose → S3 → Athena + QuickSight.




## 5. Multi-Environment Architecture

Three isolated environments share only the Route 53 hosted zone and ACM wildcard certificate. All other resources are fully duplicated per environment.

```javascript
┌──────────────────────── SHARED ────────────────────────────┐
│                                                            │
│   Route 53 Hosted Zone          ACM Wildcard Certificate   │
│   (yourdomain.dev)              (*.yourdomain.dev)         │
│                                                            │
│   ProtonMail DNS Records        GitHub OIDC Provider       │
│   (MX, SPF, DKIM, DMARC)       (account-wide)              │
│                                                            │
└────────────┬──────────────────┬──────────────┬─────────────┘
             │                  │              │
             ▼                  ▼              ▼
┌────────────────┐  ┌────────────────┐  ┌────────────────────┐
│      DEV       │  │     STAGE      │  │       PROD         │
│                │  │                │  │                    │
│ dev.domain.dev │  │stage.domain.dev│  │   domain.dev       │
│                │  │                │  │   www.domain.dev   │
│ S3 Bucket      │  │ S3 Bucket      │  │ S3 Bucket          │
│ CloudFront     │  │ CloudFront     │  │ CloudFront         │
│ API Gateway    │  │ API Gateway    │  │ API Gateway        │
│ Lambda (128MB) │  │ Lambda (128MB) │  │ Lambda (256MB)     │
│ DynamoDB       │  │ DynamoDB       │  │ DynamoDB           │
│                │  │                │  │ WAF v2             │
│ No WAF         │  │ No WAF         │  │ S3 Versioning      │
│ 7-day logs     │  │ 30-day logs    │  │ 90-day logs        │
│ No versioning  │  │ Versioning on  │  │ PITR enabled       │
└────────────────┘  └────────────────┘  └────────────────────┘
```




## 6. CI/CD Pipeline

### 6.1 Branching Strategy

The project uses a simplified Git Flow with three long-lived branches mapped to environments:

| Branch | Deploys to | Trigger | Approval |
| --- | --- | --- | --- |
| `develop` | dev.yourdomain.dev | Auto on merge | PR review (1 approver) |
| `main` | stage.yourdomain.dev | Auto on merge | PR review (1 approver) |
| `main` | yourdomain.dev | Manual gate | GitHub Environment reviewer |

Feature branches (`feature/*`, `bugfix/*`) are created from `develop` and merged back via PR. Hotfixes branch from `main` and are back-merged to `develop` after deployment.

### 6.2 Pipeline Stages

```javascript
 ┌─────────────┐
 │ Push / PR   │
 │ Merge       │
 └──────┬──────┘
        │
        ▼
 ┌─────────────┐     Build artifact is created ONCE
 │   BUILD     │     and promoted through environments.
 │             │
 │ Rust/WASM   │     1. Install Rust + wasm32 target
 │ wasm-pack   │     2. wasm-pack build --target web --release
 │ Assemble    │     3. Copy HTML/CSS/JS + WASM pkg/ into dist/
 └──────┬──────┘     4. Upload as GitHub Actions artifact
        │
        ├──── develop branch ──── ▶ DEPLOY DEV (auto)
        │                            │ S3 sync + CloudFront invalidation
        │                            │ HTML cache: 60 seconds
        │
        ├──── main branch ──────▶ DEPLOY STAGE (auto)
        │                            │ S3 sync + CloudFront invalidation
        │                            │ HTML cache: 300 seconds
        │
        └──── main branch ──────▶ DEPLOY PROD (manual approval)
                                     │ Reviewer approves in GitHub UI
                                     │ S3 sync + CloudFront invalidation
                                     │ HTML cache: 300 seconds
```

### 6.3 Authentication

The pipeline uses **GitHub OIDC** to assume AWS IAM roles — no long-lived AWS access keys are stored. Each environment has a dedicated IAM role scoped to only its S3 bucket and CloudFront distribution, using the `environment:`claim in the OIDC subject condition. The dev pipeline physically cannot access prod resources.

### 6.4 Cache Strategy

| File Type | Dev | Stage | Prod |
| --- | --- | --- | --- |
| `.html` | 60s | 5 min | 5 min |
| `.wasm` | 1 year (immutable) | 1 year (immutable) | 1 year (immutable) |
| Other assets | 24 hours | 24 hours | 24 hours |

WASM files use long immutable caches because wasm-pack generates content-hashed filenames. CloudFront invalidation runs after every deployment as a safety net.




## 7. Geo-Redundancy (Future Enhancement)

The current architecture already provides global low-latency delivery through CloudFront's 400+ edge locations. Two additional levels of origin redundancy can be added incrementally:

### Level 1 — S3 Origin Failover

Adds a replica S3 bucket in a second region (e.g., `ap-northeast-1` Tokyo) with cross-region replication. CloudFront's origin failover group automatically switches to the replica if the primary returns 5xx errors. This protects static content delivery against a full regional S3 outage.

### Level 2 — Multi-Region Backend

Deploys Lambda + API Gateway in the failover region and converts DynamoDB to a Global Table for automatic cross-region data synchronization. Route 53 latency-based routing directs API calls to the nearest region.

```javascript
                    ┌──────────────────┐
                    │    CloudFront    │
                    │  Origin Failover │
                    │  Group           │
                    └────┬────────┬────┘
                         │        │
            Primary      │        │     Failover
         ┌───────────────┘        └───────────────┐
         ▼                                        ▼
┌──────────────────┐                  ┌──────────────────┐
│  eu-central-1    │                  │ ap-northeast-1   │
│  (Frankfurt)     │                  │ (Tokyo)          │
│                  │                  │                  │
│  S3 (primary)  ◄─── Replication ──►  S3 (replica)      │
│  API Gateway     │                  │  API Gateway     │
│  Lambda          │                  │  Lambda          │
│  DynamoDB ◄──── Global Tables ────► DynamoDB           │
└──────────────────┘                  └──────────────────┘
```

Both levels are designed as bolt-on Terraform files that do not require changes to the existing infrastructure.




## 8. Cost Estimate

### 8.1 Monthly Cost (Production, Low Traffic — <50k requests/month)

| Service | Estimated Cost | Notes |
| --- | --- | --- |
| Route 53 Hosted Zone | $0.50 | Fixed monthly fee |
| Route 53 DNS Queries | \~$0.00 | Alias queries to CloudFront are free |
| CloudFront | \~$0.00 | 1 TB/month free tier (first year) |
| S3 Storage | \~$0.01 | Few MB of HTML/CSS/JS/WASM |
| S3 Requests | \~$0.01 | Mostly served from CloudFront cache |
| ACM Certificate | $0.00 | Always free |
| Lambda | \~$0.00 | 1M requests/month free tier |
| API Gateway | \~$0.00 | 1M requests/month free tier (HTTP API) |
| DynamoDB | \~$0.00 | 25 GB + 25 WCU/RCU free tier |
| WAF v2 | \~$6.00 | $5 WebACL + $1 per rule per month |
| CloudWatch Logs | \~$0.50 | Minimal log volume |
| **Total (prod)** | **\~$7/month** | **Excluding domain registration** |

### 8.2 Annual Fixed Costs

| Item | Cost |
| --- | --- |
| Domain registration (.dev) | \~$12/year (at external registrar) |
| Route 53 Hosted Zone | $6/year |
| **Total** | **\~$18/year** |

### 8.3 Multi-Environment Total

Dev and stage environments cost less because WAF is disabled. Estimated total across all three environments: **\~$10–15/month**.




## 9. Terraform Project Structure

```javascript
repository/
├── .github/
│   └── workflows/
│       └── deploy.yml                 # Multi-environment CI/CD pipeline
│
├── terraform/
│   ├── modules/
│   │   └── static-site/               # Reusable module
│   │       ├── main.tf                # S3, CloudFront, Lambda, API GW,
│   │       │                          # DynamoDB, DNS records
│   │       ├── variables.tf           # Module inputs (env, domain,
│   │       │                          # subdomain, feature toggles)
│   │       └── outputs.tf             # URLs, bucket names, distribution IDs
│   │
│   ├── shared/                        # Applied once, shared across envs
│   │   ├── main.tf                    # Route 53 zone, ACM cert
│   │   ├── cicd.tf                    # GitHub OIDC provider + IAM roles
│   │   └── email.tf                   # ProtonMail MX/SPF/DKIM/DMARC
│   │
│   └── environments/
│       ├── dev/main.tf                # Calls module: subdomain="dev"
│       ├── stage/main.tf              # Calls module: subdomain="stage"
│       └── prod/main.tf               # Calls module: subdomain=""
│
├── web/                               # Static HTML/CSS/JS
│   ├── index.html
│   ├── 404.html
│   └── ...
│
└── crates/                            # Rust crate
    ├── trellis-wasm 
        ├── Cargo.toml
        ├── Cargo.lock
        └── src/
            └── lib.rs
```

Each environment has an independent Terraform state file stored in S3, ensuring that changes to dev cannot accidentally affect prod.




## 10. Security Summary

| Layer | Mechanism |
| --- | --- |
| TLS | ACM certificate, TLS 1.2+ minimum, HTTPS enforced |
| CDN | CloudFront with OAC (no public S3 access) |
| WAF | Rate limiting, AWS Common Rules, Known Bad Inputs |
| Headers | HSTS, X-Frame-Options DENY, CSP-compatible Permissions-Policy |
| IAM | Least-privilege roles per Lambda and CI/CD environment |
| CI/CD auth | OIDC federation (no long-lived AWS credentials) |
| Data | S3 AES-256 encryption, DynamoDB encryption at rest |
| Email | SPF + DKIM + DMARC on custom domain |
| DNS | DNSSEC available via Route 53 |




## Appendix A: Deployment Sequence (Initial Setup)

1. Create S3 bucket for Terraform remote state + DynamoDB lock table
2. `cd terraform/shared && terraform init && terraform apply`
3. Note the nameserver output, configure at domain registrar
4. Add ProtonMail verification code and DKIM values, re-apply shared
5. Verify domain in ProtonMail dashboard
6. `cd terraform/environments/dev && terraform init && terraform apply`
7. `cd terraform/environments/stage && terraform init && terraform apply`
8. `cd terraform/environments/prod && terraform init && terraform apply`
9. Configure GitHub repository: environments, secrets, branch protection
10. Push to `develop` — first automated deployment to dev environment

## Appendix B: Key Decisions Log

| Decision | Choice | Rationale |
| --- | --- | --- |
| SPA framework | None | Requirement: pure HTML/CSS/JS + WASM |
| Hosting | S3 + CloudFront | Cheapest, most scalable static hosting on AWS |
| API type | HTTP API (v2) | 70% cheaper than REST API, sufficient features |
| Database | DynamoDB on-demand | Zero idle cost, single-table design |
| Certificate | ACM | Free, auto-renewing, native CloudFront integration |
| Domain DNS | Route 53 (DNS only) | `.dev` not available for registration in Route 53 |
| Email | ProtonMail | Privacy-focused, clean DNS coexistence |
| Analytics | GA4 (initial) | Zero infrastructure, mature dashboards |
| IaC | Terraform with modules | Reusable across environments, clear state separation |
| CI/CD auth | GitHub OIDC | No stored secrets, per-environment role isolation |
| WAF | Prod only | Cost optimization; \~$6/month saved per non-prod env |
| WASM toolchain | wasm-pack --target web | Native ES module output, no bundler required |
