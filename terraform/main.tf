data "aws_ssm_parameter" "postgres_uri" {
  name = "WedPostgresUriDev"
}

resource "null_resource" "copy_build" {
  provisioner "local-exec" {
    command     = "cp target/aarch64-unknown-linux-musl/release/wedding_funcs bootstrap && zip bootstrap.zip bootstrap"
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

resource "aws_api_gateway_method" "options_method" {
  rest_api_id   = aws_api_gateway_rest_api.wedding_api.id
  resource_id   = aws_api_gateway_resource.api_resource.id
  http_method   = "OPTIONS"
  authorization = "NONE"
}

resource "aws_api_gateway_method_response" "cors_method_response" {
    rest_api_id   = aws_api_gateway_rest_api.wedding_api.id
    resource_id   = aws_api_gateway_resource.api_resource.id
    http_method   = aws_api_gateway_method.options_method.http_method
    status_code   = 200
    response_parameters = {
        "method.response.header.Access-Control-Allow-Headers" = true,
        "method.response.header.Access-Control-Allow-Methods" = true,
        "method.response.header.Access-Control-Allow-Origin" = true
    }
}

resource "aws_api_gateway_integration" "cors_integration" {
  rest_api_id             = aws_api_gateway_rest_api.wedding_api.id
  resource_id             = aws_api_gateway_resource.api_resource.id
  http_method             = aws_api_gateway_method.options_method.http_method
  type                    = "MOCK"
  request_templates = {
    "application/json" = jsonencode({
      statusCode = 200
    })
  }
}

resource "aws_api_gateway_integration_response" "cors_integration_response" {
  rest_api_id             = aws_api_gateway_rest_api.wedding_api.id
  resource_id             = aws_api_gateway_resource.api_resource.id
  http_method             = aws_api_gateway_method.options_method.http_method
  status_code             = 200
  response_parameters = {
        "method.response.header.Access-Control-Allow-Origin" = "'*'",
        "method.response.header.Access-Control-Allow-Headers" = "'Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token'",
        "method.response.header.Access-Control-Allow-Methods" = "'GET,OPTIONS,POST,PUT'"
  }
}

resource "aws_api_gateway_integration" "api_integration" {
  rest_api_id             = aws_api_gateway_rest_api.wedding_api.id
  resource_id             = aws_api_gateway_resource.api_resource.id
  http_method             = aws_api_gateway_method.api_method.http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.wedding_func.invoke_arn
}

resource "aws_api_gateway_deployment" "wedding_deployment" {
  rest_api_id = aws_api_gateway_rest_api.wedding_api.id

  triggers = {
    redeployment = timestamp()
  }

  lifecycle {
    create_before_destroy = true
  }

  depends_on = [aws_api_gateway_rest_api.wedding_api]
}

resource "aws_api_gateway_stage" "PROD" {
  deployment_id = aws_api_gateway_deployment.wedding_deployment.id
  rest_api_id   = aws_api_gateway_rest_api.wedding_api.id
  stage_name    = "PROD"
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
  environment {
    variables = {
      WED_POSTGRES_URI = data.aws_ssm_parameter.postgres_uri.value
    }
  }
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
