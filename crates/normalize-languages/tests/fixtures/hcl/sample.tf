# Sample Terraform (HCL) configuration

terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

variable "region" {
  type        = string
  description = "AWS region to deploy resources into"
  default     = "us-east-1"
}

variable "instance_count" {
  type        = number
  description = "Number of EC2 instances to create"
  default     = 1
}

variable "tags" {
  type        = map(string)
  description = "Tags to apply to all resources"
  default     = {}
}

module "vpc" {
  source = "./modules/vpc"
  region = var.region
  tags   = var.tags
}

resource "aws_instance" "web" {
  count         = var.instance_count
  ami           = data.aws_ami.ubuntu.id
  instance_type = "t3.micro"

  tags = merge(var.tags, {
    Name = "web-${count.index}"
  })
}

data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"]

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-*-22.04-amd64-server-*"]
  }
}

output "instance_ids" {
  value       = aws_instance.web[*].id
  description = "IDs of the created EC2 instances"
}

locals {
  # Conditional (ternary) expression — branch
  instance_type = var.instance_count > 1 ? "m5.large" : "t3.micro"

  # for-expression comprehensions — loop-like constructs
  instance_names = [for i in range(var.instance_count) : "web-${i}"]
  instance_map   = { for i in range(var.instance_count) : i => "web-${i}" if i > 0 }
}

output "vpc_id" {
  value       = module.vpc.id
  description = "ID of the created VPC"
}

locals {
  env_prefix = "prod"
  full_name  = "${local.env_prefix}-web"
}
