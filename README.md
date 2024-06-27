# S3 Streaming API

This is a demonstration of how to create an API that can upload file to S3 using streaming.
This API also sends a notification to a kafka topic whenever a file is uploaded to S3 .

## Requirements

Digitalocean Spaces (for S3 storage)
TODO: Kafka (for notification)


## Setup infra

Go to the directory [`infra`](./infra) 

### Prepare a `.tfvars` file with the following variables
```
spaces_secret_key = "<your secret key>"
spaces_access_key = "<your access key>"
```
### Run the following command to setup the infra

```bash
terraform apply -var-file=infra/dev.tfvars
```

## Usage

You must have the following environment variables

``` bash
export AWS_ACCESS_KEY_ID="<access key>"
export AWS_SECRET_ACCESS_KEY="<the secret key>"
```

For now it is _fixed to Digitalocean Spaces region SGP1._

