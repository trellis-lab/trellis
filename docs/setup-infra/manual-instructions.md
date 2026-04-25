# Manual Setup Instructions — trellislab.net AWS Deployment

## Step 1 — Install AWS CLI

Download and install: https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html

Verify:
```bash
aws --version
```

---

## Step 2 — Create IAM User for Terraform

This user runs Terraform locally. It is **not** the GitHub Actions identity (that gets created by Terraform itself).

1. AWS Console → **IAM** → **Users** → **Create user**
2. Username: `terraform-admin`
3. **Permissions** tab → **Attach policies directly** → check `AdministratorAccess`
   > Note: AdministratorAccess is the pragmatic choice here. Terraform needs to create IAM roles, CloudFront, ACM, Route 53, S3 — scoping it down is a large policy to maintain for a single-person project.
4. **Create user** → open the user → **Security credentials** tab
5. **Create access key** → use case: **Command Line Interface (CLI)**
6. Copy **Access Key ID** and **Secret Access Key** — shown only once

---

## Step 3 — Configure AWS CLI

```bash
aws configure --profile trellislab
```

Enter when prompted:
```
AWS Access Key ID:     [paste from step 2]
AWS Secret Access Key: [paste from step 2]
Default region:        eu-central-1
Default output format: json
```

Verify it works:
```bash
aws sts get-caller-identity --profile trellislab
```

Expected output: your AWS account ID and `terraform-admin` ARN.

Set profile for the current shell session (do this before every Terraform run):
```bash
export AWS_PROFILE=trellislab
```

On Windows PowerShell:
```powershell
$env:AWS_PROFILE = "trellislab"
```

---

## Step 4 — Run Bootstrap (one-time only)

This creates the S3 bucket and DynamoDB table that all other Terraform state lives in. Uses **local** state (no remote backend yet).

```bash
cd terraform/bootstrap
terraform init
terraform apply
```

Type `yes` when prompted. Takes ~30 seconds.

Verify in AWS Console:
- S3 → bucket `trellislab-terraform-state` exists
- DynamoDB → table `terraform-state-locks` exists

---

## Step 5 — Apply Shared Terraform (two passes)

ACM cert validation requires Route 53 to be authoritative, which requires the GoDaddy nameserver switch first. Two-pass approach avoids a 75-minute timeout.

```bash
cd terraform/shared
terraform init

# Pass 1: create hosted zone only, get NS values
terraform apply -target=aws_route53_zone.main
terraform output name_servers
```

Output looks like:
```
name_servers = [
  "ns-123.awsdns-45.com",
  "ns-678.awsdns-90.net",
  "ns-111.awsdns-22.co.uk",
  "ns-999.awsdns-55.org",
]
```

**At GoDaddy** → Domains → trellislab.net → **Nameservers** → **Enter my own nameservers** → paste all 4.

Wait for propagation. Verify with:
```bash
nslookup -type=NS trellislab.net 8.8.8.8
```

When you see the Route 53 NS values (not GoDaddy's `ns55/ns56.domaincontrol.com`), continue.

```bash
# Pass 2: create ACM cert + ProtonMail DNS records
terraform apply
```

ACM cert validates in ~5–15 min once Route 53 is authoritative. `terraform apply` waits automatically.

---

## Step 6 — Apply Prod Terraform

```bash
cd terraform/environments/prod
terraform init
terraform apply
terraform output
```

Note these two output values — needed in GitHub:
- `cloudfront_distribution_id`
- `github_actions_role_arn`

---

## Step 7 — Configure GitHub Environment

1. GitHub → `trellis-mermaid/trellis` → **Settings** → **Environments** → **New environment**: `prod`
2. Under **Environment variables** (not secrets), add:
   - `CLOUDFRONT_DISTRIBUTION_ID` = value from `terraform output`
   - `IAM_ROLE_ARN` = value from `terraform output`
3. Optionally add a **Required reviewer** under Protection rules for manual approval before prod deploys

---

## Step 8 — First Deployment

Push to `main` — CI/CD pipeline runs automatically:
1. Builds WASM from `crates/trellis-wasm`
2. Syncs `web/` + WASM to S3
3. Invalidates CloudFront cache

Smoke test:
- `https://trellislab.net` loads
- WASM diagram renderer works
- Send a test email to verify ProtonMail still delivers
