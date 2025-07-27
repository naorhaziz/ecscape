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

resource "aws_iam_role" "high_priv_role" {
  name = "high-priv-role"

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

resource "aws_iam_role_policy_attachment" "high_priv_role_attachment" {
  role       = aws_iam_role.high_priv_role.name
  policy_arn = "arn:aws:iam::aws:policy/AdministratorAccess"
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

resource "aws_ecs_task_definition" "high_priv_task" {
  family             = "high-priv-task"
  network_mode       = "host"
  task_role_arn      = aws_iam_role.high_priv_role.arn
  execution_role_arn = "arn:aws:iam::${data.aws_caller_identity.current.account_id}:role/ecsTaskExecutionRole"

  container_definitions = jsonencode([
    {
      name       = "high-priv"
      image      = "ubuntu:latest"
      entryPoint = ["sleep", "infinity"]
      essential  = true
      memory     = 512
      cpu        = 256
    }
  ])
}

resource "aws_ecs_service" "high_priv_service" {
  name            = "high-priv-service"
  cluster         = aws_ecs_cluster.ecscape.id
  task_definition = aws_ecs_task_definition.high_priv_task.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ecscape_capacity_provider.name
    weight            = 100
    base              = 1
  }

  depends_on = [aws_autoscaling_group.ecs_asg]
}

