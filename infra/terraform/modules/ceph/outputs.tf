output "bucket_ids" {
  description = "Map of environment to bucket ID"
  value       = { for env, bucket in aws_s3_bucket.app_distribution : env => bucket.id }
}

output "bucket_arns" {
  description = "Map of environment to bucket ARN"
  value       = { for env, bucket in aws_s3_bucket.app_distribution : env => bucket.arn }
}

output "bucket_names" {
  description = "Map of environment to bucket name"
  value       = { for env, bucket in aws_s3_bucket.app_distribution : env => bucket.bucket }
}
