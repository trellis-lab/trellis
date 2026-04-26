"""
Daily CloudFront log repartitioner.

Copies flat cf-logs/<dist>.<YYYY-MM-DD-HH>.<hash>.gz files into
cf-logs-partitioned/year=YYYY/month=MM/day=DD/ so Athena can use
partition projection and skip scanning unrelated dates.

Triggered by EventBridge Scheduler at 02:00 UTC each day.
Processes yesterday's logs by default; override by passing {"date": "YYYY-MM-DD"}.

Idempotent: copying an already-copied file is a no-op on S3 (same ETag).
"""

import boto3
import re
import os
import logging
from datetime import datetime, timedelta, timezone

log = logging.getLogger()
log.setLevel(logging.INFO)

s3 = boto3.client("s3")

BUCKET = os.environ["LOG_BUCKET"]
SOURCE_PREFIX = "cf-logs/"
DEST_PREFIX = "cf-logs-partitioned/"

# E1XYZABCDEF123.2026-04-26-14.a1b2c3d4.gz
FILENAME_RE = re.compile(r"^[^/]+/([\w-]+)\.(\d{4})-(\d{2})-(\d{2})-\d{2}\.\w+\.gz$")


def lambda_handler(event, context):
    target_date = event.get("date") or (
        datetime.now(timezone.utc) - timedelta(days=1)
    ).strftime("%Y-%m-%d")

    year, month, day = target_date.split("-")
    dest_prefix = f"{DEST_PREFIX}year={year}/month={month}/day={day}/"

    log.info("Repartitioning logs for %s → %s", target_date, dest_prefix)

    paginator = s3.get_paginator("list_objects_v2")
    moved = skipped = errors = 0

    for page in paginator.paginate(Bucket=BUCKET, Prefix=SOURCE_PREFIX):
        for obj in page.get("Contents", []):
            key = obj["Key"]
            m = FILENAME_RE.match(key)
            if not m:
                continue

            file_year, file_month, file_day = m.group(2), m.group(3), m.group(4)
            if (file_year, file_month, file_day) != (year, month, day):
                continue

            filename = key.split("/")[-1]
            dest_key = dest_prefix + filename

            try:
                s3.copy_object(
                    Bucket=BUCKET,
                    CopySource={"Bucket": BUCKET, "Key": key},
                    Key=dest_key,
                    # Preserve original server-side encryption
                    ServerSideEncryption="AES256",
                )
                moved += 1
                log.debug("Copied %s → %s", key, dest_key)
            except Exception as exc:
                log.error("Failed to copy %s: %s", key, exc)
                errors += 1

    log.info("Done: moved=%d skipped=%d errors=%d", moved, skipped, errors)

    if errors:
        raise RuntimeError(f"{errors} files failed to copy — check CloudWatch logs")

    return {"date": target_date, "files_moved": moved}
