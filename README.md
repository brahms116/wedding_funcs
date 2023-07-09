# Lambda function for wedding website

This repo contains the api for the wedding website frontend to allow invitees to rsvp

### Aws services

This repo contains the code for an AWS Lambda function in rust. It is used with a AWS Api Gateway Proxy.
The function connects with a Postgres db using an connection uri set as an environment variable. The details of these services
can be found in the terraform configuration in the `terraform/modules` directory.

### Deployment

Currently, this function and api can only be deployed manually.

The function uses a ecr docker image, specified by the terraform variable `image_uri`.

The tag for production is `latest` and `dev` for develop.

Build the image locally and push it into ecr.

There are two environements configured under `terraform/dev` and `terraform/prod`.
To deploy for an environment, navigate into the respective directory, then run...

```bash
terraform init
terraform plan
terraform apply
```

### TODOs

- Document api endpoints
- Terraform deployment should be done inside github actions
- Docker container build should be done inside github actions
