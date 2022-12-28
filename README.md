# Lambda function for wedding website

This repo contains the api for the wedding website frontend to allow invitees to rsvp

### Aws services

This repo contains the code for an AWS Lambda function in rust. It is used with a AWS Api Gateway Proxy.
The function connects with a Postgres db using an connection uri set as an environment variable. The details of these services
can be found in the terraform configuration in the `terraform/modules` directory.

### Deployment

Currently, this function and api can only be deployed manually. There are two environements configured under `terraform/dev` and `terraform/prod`.
Terraform currently looks for a binary `target/aarch64-unknown-linux-musl/release/bootstrap` to archive and upload
To deploy for an environment, navigate into the respective directory, then run...

```bash
terraform init
terraform plan
terraform apply
```

### TODOs

- Document api endpoints
- Terraform deployment should be done inside github actions
