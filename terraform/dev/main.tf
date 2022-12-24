provider "aws" {
  region = "ap-southeast-2"
}

terraform {
  backend "s3" {
    key    = "wedding-api/dev.tfstate"
    bucket = "pastureen-tf-state-store"
    region = "ap-southeast-2"
  }
}

module "deployment" {
  source      = "../modules"
  environment = "dev"
}
