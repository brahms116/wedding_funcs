variable "environment" {
  type        = string
  description = "The name of the environment terraform should build for"
}

data "aws_ssm_parameter" "postgres_uri" {
  name = "wedding-postgres-uri-${var.environment}"
}

resource "aws_api_gateway_rest_api" "wedding_api" {
  name = "wedding-api-${var.environment}"
}

resource "aws_api_gateway_resource" "api_resource" {
  path_part   = "api"
  parent_id   = aws_api_gateway_rest_api.wedding_api.root_resource_id
  rest_api_id = aws_api_gateway_rest_api.wedding_api.id
  depends_on  = [aws_api_gateway_rest_api.wedding_api]
}

resource "aws_api_gateway_method" "api_method" {
  rest_api_id   = aws_api_gateway_rest_api.wedding_api.id
  resource_id   = aws_api_gateway_resource.api_resource.id
  http_method   = "POST"
  authorization = "NONE"
  depends_on    = [aws_api_gateway_resource.api_resource]
}

resource "aws_api_gateway_method" "options_method" {
  rest_api_id   = aws_api_gateway_rest_api.wedding_api.id
  resource_id   = aws_api_gateway_resource.api_resource.id
  http_method   = "OPTIONS"
  authorization = "NONE"
  depends_on    = [aws_api_gateway_resource.api_resource]
}

resource "aws_api_gateway_method_response" "cors_method_response" {
  rest_api_id = aws_api_gateway_rest_api.wedding_api.id
  resource_id = aws_api_gateway_resource.api_resource.id
  http_method = aws_api_gateway_method.options_method.http_method
  status_code = 200
  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true,
    "method.response.header.Access-Control-Allow-Methods" = true,
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
  depends_on = [aws_api_gateway_method.options_method]
}

resource "aws_api_gateway_integration" "cors_integration" {
  rest_api_id = aws_api_gateway_rest_api.wedding_api.id
  resource_id = aws_api_gateway_resource.api_resource.id
  http_method = aws_api_gateway_method.options_method.http_method
  type        = "MOCK"
  request_templates = {
    "application/json" = jsonencode({
      statusCode = 200
    })
  }
  depends_on = [aws_api_gateway_method_response.cors_method_response]
}

resource "aws_api_gateway_integration_response" "cors_integration_response" {
  rest_api_id = aws_api_gateway_rest_api.wedding_api.id
  resource_id = aws_api_gateway_resource.api_resource.id
  http_method = aws_api_gateway_method.options_method.http_method
  status_code = 200
  response_parameters = {
    "method.response.header.Access-Control-Allow-Origin"  = "'*'",
    "method.response.header.Access-Control-Allow-Headers" = "'Content-Type,X-Amz-Date,Authorization,X-Api-Key,X-Amz-Security-Token'",
    "method.response.header.Access-Control-Allow-Methods" = "'GET,OPTIONS,POST,PUT'"
  }
  depends_on = [aws_api_gateway_method.options_method]
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

resource "aws_api_gateway_integration" "api_integration" {
  rest_api_id             = aws_api_gateway_rest_api.wedding_api.id
  resource_id             = aws_api_gateway_resource.api_resource.id
  http_method             = aws_api_gateway_method.api_method.http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.wedding_func.invoke_arn
  depends_on              = [aws_api_gateway_method.api_method]
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
  function_name = "wedding-api-function-${var.environment}"
  role          = aws_iam_role.role.arn
  image_uri     = "476915837883.dkr.ecr.ap-southeast-2.amazonaws.com/wedding-funcs-manual:latest"
  package_type  = "Image"
  architectures = ["arm64"]
  environment {
    variables = {
      SSL_CERT_PATH    = "/etc/ssl/certs/ca-certificates.crt",
      WED_POSTGRES_URI = data.aws_ssm_parameter.postgres_uri.value
    }
  }
}

resource "aws_iam_role" "role" {
  name = "lambda-execution-role-${var.environment}"

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

resource "aws_iam_role_policy_attachment" "basic_execution_attachment" {
  role       = aws_iam_role.role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}
