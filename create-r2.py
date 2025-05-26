#!/usr/bin/env python3
import os
import sys
import uuid
from datetime import datetime
import boto3
from botocore.config import Config


def load_env(filepath='/usr/local/bin/.env'):
    env_vars = {}
    with open(filepath, 'r') as file:
        for line in file:
            line = line.strip() # skip empty lines
            if '=' in line:
                key, value = line.split('=', 1)
                key = key.strip()
                value = value.strip()
                env_vars[key] = value
    return env_vars


def main():
    
    # Get r2 info from .env
    config = load_env()

    # Get input file from command line
    input_file = sys.argv[1]
    
    # Configuration
    access_key = config.get('ACCESS_KEY')
    secret_access_key = config.get('SECRET_ACCESS_KEY') 
    endpoint_url = config.get('ENDPOINT_URL')
    public_domain = config.get('PUBLIC_DOMAIN')
    bucket_name = config.get('BUCKET_NAME')
     
    # Generate unique object key
    date_prefix = datetime.now().strftime("%Y%m%d")
    object_key = f"mp3/{date_prefix}/{uuid.uuid4()}.mp3"
    
    # Initialize S3 client for R2
    s3 = boto3.client('s3',
        endpoint_url=endpoint_url,
        aws_access_key_id=access_key,
        aws_secret_access_key=secret_access_key,
        config=Config(signature_version='s3v4')
    )
    
    try:
        # Upload file
        s3.upload_file(input_file, bucket_name, object_key)
        print(f"{public_domain}/{object_key}")
    except Exception as e:
        sys.stderr.write(f"Error uploading file: {str(e)}")
        sys.exit(1)

if __name__ == "__main__":
    main()
