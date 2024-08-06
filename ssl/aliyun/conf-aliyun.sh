#!/bin/bash

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <AccessKeyId> <AccessKeySecret>"
  exit 1
fi

ACCESS_KEY_ID=$1
ACCESS_KEY_SECRET=$2


PROFILE_NAME="akProfile"
MODE="AK"
REGION="cn-hangzhou"

sudo aliyun configure set \
  --profile $PROFILE_NAME \
  --mode $MODE \
  --region $REGION \
  --access-key-id $ACCESS_KEY_ID \
  --access-key-secret $ACCESS_KEY_SECRET

sudo ls /root/.aliyun
echo "Configuration completed successfully."
