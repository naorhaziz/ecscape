provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-2"
}

data "aws_caller_identity" "current" {}

resource "aws_iam_policy" "ecscape_policy" {
  name        = "ecscape-policy"
  description = "Policy that denies all actions"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Deny"
        Action   = "*"
        Resource = "*"
      }
    ]
  })
}

resource "aws_iam_role" "ecscape_role" {
  name = "ecscape-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "ecscape_role_attachment" {
  role       = aws_iam_role.ecscape_role.name
  policy_arn = aws_iam_policy.ecscape_policy.arn
}

resource "aws_iam_role" "s3_control_role" {
  name = "s3-control-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "s3_control_role_attachment" {
  role       = aws_iam_role.s3_control_role.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonS3FullAccess"
}

# Create the secret
resource "aws_secretsmanager_secret" "db_secret" {
  name        = "db-secret"
  description = "Database secret for demo"
}

resource "aws_secretsmanager_secret_version" "db_secret_version" {
  secret_id     = aws_secretsmanager_secret.db_secret.id
  secret_string = "SuperSecretPassword"
}

# Custom execution role with permissions to read the specific secret
resource "aws_iam_role" "secret_execution_role" {
  name = "secret-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })
}

# Base ECS task execution permissions
resource "aws_iam_role_policy_attachment" "secret_execution_role_base" {
  role       = aws_iam_role.secret_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

# Custom policy to read the specific secret
resource "aws_iam_policy" "read_db_secret_secret" {
  name        = "read-db-password-secret"
  description = "Policy to read DB_SECRET secret"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "secretsmanager:GetSecretValue"
        ]
        Resource = aws_secretsmanager_secret.db_secret.arn
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "secret_execution_role_secret_policy" {
  role       = aws_iam_role.secret_execution_role.name
  policy_arn = aws_iam_policy.read_db_secret_secret.arn
}

# S3 bucket for demonstration
resource "aws_s3_bucket" "s3_bucket" {
  bucket = "blackhat-las-vegas-2025"
}

resource "aws_s3_bucket_public_access_block" "s3_bucket" {
  bucket = aws_s3_bucket.s3_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_ecs_cluster" "ecscape" {
  name = "ecscape"
}

data "aws_vpc" "default" {
  default = true
}

data "aws_subnets" "default" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default.id]
  }
}

resource "aws_autoscaling_group" "ecs_asg" {
  name                = "ecscape-asg"
  vpc_zone_identifier = data.aws_subnets.default.ids
  min_size            = 1
  max_size            = 1
  desired_capacity    = 1

  launch_template {
    id      = aws_launch_template.ecs_launch_template.id
    version = "$Latest"
  }

  tag {
    key                 = "AmazonECSManaged"
    value               = true
    propagate_at_launch = false
  }
}

resource "aws_ecs_capacity_provider" "ecscape_capacity_provider" {
  name = "escape-cp"

  auto_scaling_group_provider {
    auto_scaling_group_arn = aws_autoscaling_group.ecs_asg.arn

    managed_scaling {
      maximum_scaling_step_size = 1
      minimum_scaling_step_size = 1
      status                    = "ENABLED"
      target_capacity           = 100
    }

    managed_termination_protection = "DISABLED"
  }
}

resource "aws_ecs_cluster_capacity_providers" "ecscape" {
  cluster_name = aws_ecs_cluster.ecscape.name

  capacity_providers = [aws_ecs_capacity_provider.ecscape_capacity_provider.name]

  default_capacity_provider_strategy {
    base              = 1
    weight            = 100
    capacity_provider = aws_ecs_capacity_provider.ecscape_capacity_provider.name
  }
}

data "aws_ami" "ecs_optimized" {
  most_recent = true
  owners      = ["amazon"]

  filter {
    name   = "name"
    values = ["amzn2-ami-ecs-hvm-*-x86_64-ebs"]
  }
}

data "aws_iam_instance_profile" "ecs_instance_profile" {
  name = "ecsInstanceRole"
}

resource "aws_iam_role" "ecs_instance_role" {
  name = "ecscape-ecs-instance-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "ecs_instance_role_policy" {
  role       = aws_iam_role.ecs_instance_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonEC2ContainerServiceforEC2Role"
}

resource "aws_iam_role_policy_attachment" "ecs_instance_ssm_policy" {
  role       = aws_iam_role.ecs_instance_role.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_instance_profile" "ecs_instance_profile" {
  name = "ecscape-ecs-instance-profile"
  role = aws_iam_role.ecs_instance_role.name
}

resource "aws_launch_template" "ecs_launch_template" {
  name_prefix   = "ecscape-launch-template-"
  image_id      = data.aws_ami.ecs_optimized.id
  instance_type = "c5.large"

  iam_instance_profile {
    name = aws_iam_instance_profile.ecs_instance_profile.name
  }

  user_data = base64encode(<<-EOF
    #!/bin/bash
    echo ECS_CLUSTER=ecscape >> /etc/ecs/ecs.config
  EOF
  )
}

resource "aws_ecs_task_definition" "ecscape_task" {
  family        = "ecscape-task"
  network_mode  = "host"
  task_role_arn = aws_iam_role.ecscape_role.arn

  container_definitions = jsonencode([
    {
      name       = "ecscape"
      image      = "ghcr.io/naorhaziz/ecscape:latest"
      entryPoint = ["sleep", "infinity"]
      essential  = true
      memory     = 512
      cpu        = 256
    }
  ])
}

resource "aws_ecs_service" "ecscape_service" {
  name            = "ecscape-service"
  cluster         = aws_ecs_cluster.ecscape.id
  task_definition = aws_ecs_task_definition.ecscape_task.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ecscape_capacity_provider.name
    weight            = 100
    base              = 1
  }

  depends_on = [aws_autoscaling_group.ecs_asg]
}

resource "aws_ecs_task_definition" "s3_control_task" {
  family             = "s3-control-task"
  network_mode       = "host"
  task_role_arn      = aws_iam_role.s3_control_role.arn

  container_definitions = jsonencode([
    {
      name       = "s3-control"
      image      = "ubuntu:latest"
      entryPoint = ["sleep", "infinity"]
      essential  = true
      memory     = 512
      cpu        = 256
    }
  ])
}

resource "aws_ecs_service" "s3_control_service" {
  name            = "s3-control-service"
  cluster         = aws_ecs_cluster.ecscape.id
  task_definition = aws_ecs_task_definition.s3_control_task.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ecscape_capacity_provider.name
    weight            = 100
    base              = 1
  }

  depends_on = [aws_autoscaling_group.ecs_asg]
}

resource "aws_ecs_task_definition" "database_task" {
  family             = "database-task"
  network_mode       = "host"
  execution_role_arn = aws_iam_role.secret_execution_role.arn

  container_definitions = jsonencode([
    {
      name       = "database-app"
      image      = "ubuntu:latest"
      entryPoint = ["sleep", "infinity"]
      essential  = true
      memory     = 512
      cpu        = 256
      
      # Secret from Secrets Manager exposed as environment variable
      secrets = [
        {
          name      = "DB_SECRET"
          valueFrom = aws_secretsmanager_secret.db_secret.arn
        }
      ]
    }
  ])
}

resource "aws_ecs_service" "database_service" {
  name            = "database-service"
  cluster         = aws_ecs_cluster.ecscape.id
  task_definition = aws_ecs_task_definition.database_task.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ecscape_capacity_provider.name
    weight            = 100
    base              = 1
  }

  depends_on = [aws_autoscaling_group.ecs_asg]
}

resource "aws_s3_bucket" "cloudtrail_bucket" {
  bucket        = "ecscape-cloudtrail-${random_id.bucket_suffix.hex}"
  force_destroy = true
}

resource "random_id" "bucket_suffix" {
  byte_length = 4
}

resource "aws_s3_bucket_public_access_block" "cloudtrail_bucket" {
  bucket = aws_s3_bucket.cloudtrail_bucket.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_policy" "cloudtrail_bucket_policy" {
  bucket = aws_s3_bucket.cloudtrail_bucket.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AWSCloudTrailAclCheck"
        Effect = "Allow"
        Principal = {
          Service = "cloudtrail.amazonaws.com"
        }
        Action   = "s3:GetBucketAcl"
        Resource = aws_s3_bucket.cloudtrail_bucket.arn
      },
      {
        Sid    = "AWSCloudTrailWrite"
        Effect = "Allow"
        Principal = {
          Service = "cloudtrail.amazonaws.com"
        }
        Action   = "s3:PutObject"
        Resource = "${aws_s3_bucket.cloudtrail_bucket.arn}/*"
        Condition = {
          StringEquals = {
            "s3:x-amz-acl" = "bucket-owner-full-control"
          }
        }
      }
    ]
  })
}

# CloudWatch Log Group for CloudTrail
resource "aws_cloudwatch_log_group" "cloudtrail_logs" {
  name              = "/aws/cloudtrail/ecscape"
  retention_in_days = 7
}

# IAM Role for CloudTrail to write to CloudWatch
resource "aws_iam_role" "cloudtrail_role" {
  name = "ecscape-cloudtrail-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "cloudtrail.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy" "cloudtrail_logs_policy" {
  name = "ecscape-cloudtrail-logs-policy"
  role = aws_iam_role.cloudtrail_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents",
          "logs:DescribeLogGroups",
          "logs:DescribeLogStreams"
        ]
        Resource = "arn:aws:logs:${var.aws_region}:${data.aws_caller_identity.current.account_id}:log-group:/aws/cloudtrail/ecscape*"
      }
    ]
  })
}

# CloudTrail itself
resource "aws_cloudtrail" "ecscape_trail" {
  name           = "ecscape-trail"
  s3_bucket_name = aws_s3_bucket.cloudtrail_bucket.bucket

  # Log to CloudWatch Logs
  cloud_watch_logs_group_arn = "${aws_cloudwatch_log_group.cloudtrail_logs.arn}:*"
  cloud_watch_logs_role_arn  = aws_iam_role.cloudtrail_role.arn

  # Log data events for S3
  event_selector {
    read_write_type                 = "All"
    include_management_events       = true
    exclude_management_event_sources = []

    # Log S3 data events
    data_resource {
      type   = "AWS::S3::Object"
      values = ["${aws_s3_bucket.s3_bucket.arn}/*"]
    }
  }

  depends_on = [aws_s3_bucket_policy.cloudtrail_bucket_policy]
}

