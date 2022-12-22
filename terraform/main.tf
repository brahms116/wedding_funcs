resource "null_resource" "copy_build" {
  provisioner "local-exec" {
    command = "cp target/aarch64-unknown-linux-musl/release/wedding_funcs bootstrap && zip bootstrap.zip bootstrap"
    working_dir = "../"
  }

  triggers = {
    rebuild = timestamp()
  }
}

provider "aws" {
  region = "ap-southeast-2"
}

resource "aws_api_gateway_rest_api" "wedding_api" {
  name = "WeddingApi"
}

resource "aws_api_gateway_resource" "api_resource" {
  path_part   = "api"
  parent_id   = aws_api_gateway_rest_api.wedding_api.root_resource_id
  rest_api_id = aws_api_gateway_rest_api.wedding_api.id
}

resource "aws_api_gateway_method" "api_method" {
  rest_api_id   = aws_api_gateway_rest_api.wedding_api.id
  resource_id   = aws_api_gateway_resource.api_resource.id
  http_method   = "POST"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "api_integration" {
  rest_api_id             = aws_api_gateway_rest_api.wedding_api.id
  resource_id             = aws_api_gateway_resource.api_resource.id
  http_method             = aws_api_gateway_method.api_method.http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.wedding_func.invoke_arn
}

resource "aws_lambda_permission" "apigw_lambda" {
  statement_id  = "AllowExecutionFromLambda"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.wedding_func.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.wedding_api.execution_arn}/*/POST/api"
}

resource "aws_lambda_function" "wedding_func" {
  function_name    = "WeddingFunction"
  filename         = "../bootstrap.zip"
  role             = aws_iam_role.role.arn
  source_code_hash = timestamp()
  handler          = "bootstrap"
  runtime          = "provided.al2"
  architectures    = ["arm64"]
  depends_on = [null_resource.copy_build]
}

resource "aws_iam_role" "role" {
  name = "myrole"

  assume_role_policy = <<POLICY
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": "sts:AssumeRole",
      "Principal": {
        "Service": "lambda.amazonaws.com"
      },
      "Effect": "Allow",
      "Sid": ""
    }
  ]
}
POLICY
}
