terraform {
  required_version = ">= 1.5"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

# S3-compatible provider for Ceph RGW
provider "aws" {
  alias = "ceph"

  endpoints {
    s3 = var.ceph_s3_endpoint
  }

  access_key = var.ceph_access_key
  secret_key = var.ceph_secret_key
  region     = "default"

  # Ceph RGW compatibility
  skip_credentials_validation = true
  skip_metadata_api_check     = true
  skip_region_validation      = true
  skip_requesting_account_id  = true
  s3_use_path_style           = true
}

# Create buckets per environment
resource "aws_s3_bucket" "app_distribution" {
  provider = aws.ceph

  for_each = toset(var.environments)

  bucket = each.value == "prod" ? var.bucket_prefix : "${var.bucket_prefix}-${each.value}"

  tags = merge(var.tags, {
    Environment = each.value
    Service     = "app-registry"
    ManagedBy   = "terraform"
  })
}

# Versioning
resource "aws_s3_bucket_versioning" "app_distribution" {
  provider = aws.ceph

  for_each = aws_s3_bucket.app_distribution

  bucket = each.value.id

  versioning_configuration {
    status = "Enabled"
  }
}

# CORS configuration
resource "aws_s3_bucket_cors_configuration" "app_distribution" {
  provider = aws.ceph

  for_each = aws_s3_bucket.app_distribution

  bucket = each.value.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET", "HEAD"]
    allowed_origins = var.cors_allowed_origins
    expose_headers  = ["ETag", "Content-Length", "Content-Type"]
    max_age_seconds = 3600
  }
}

# Lifecycle rules for non-current version cleanup
resource "aws_s3_bucket_lifecycle_configuration" "app_distribution" {
  provider = aws.ceph

  for_each = aws_s3_bucket.app_distribution

  bucket = each.value.id

  rule {
    id     = "cleanup-noncurrent-versions"
    status = "Enabled"

    noncurrent_version_expiration {
      noncurrent_days = var.lifecycle_expiration_days
    }
  }
}

# Server-side encryption
resource "aws_s3_bucket_server_side_encryption_configuration" "app_distribution" {
  provider = aws.ceph

  for_each = aws_s3_bucket.app_distribution

  bucket = each.value.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# Block public access
resource "aws_s3_bucket_public_access_block" "app_distribution" {
  provider = aws.ceph

  for_each = aws_s3_bucket.app_distribution

  bucket = each.value.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}
