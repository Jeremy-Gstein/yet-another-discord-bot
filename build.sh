#!/usr/bin/bash

BUILD_TAG="bday"
CONTAINER_NAME="bday_app"

build_dockerfile() {
  docker build -t "$BUILD_TAG" .
}

run_docker() {
  docker run -itd --env-file .env --name "$CONTAINER_NAME" --rm bday:latest
}

check_docker() {
  if docker inspect "$CONTAINER_NAME" &> /dev/null; then
    echo "Container $CONTAINER_NAME exists."
    docker run -itd --env-file .env --name "tmp-$CONTAINER_NAME" --rm bday:latest
    sleep 5
    docker rm -f "$CONTAINER_NAME"
    sleep 5
    run_docker
    sleep 5
    docker rm -f "tmp-$CONTAINER_NAME"
  else
    echo "Container $CONTAINER_NAME not found."
    run_docker
  fi
}

printf "build.sh for yet another discord bot (yadb | bday)\n"
printf "Usage:  ./build.sh (alias for docker ps atm)\n"
printf "\t-b (build app with 1 buffer/tmp container app)\n"
if [ "$1" == "-b" ]; then
  echo "Building $BUILD_TAG"
  build_dockerfile
  check_docker
else
  echo "Running $BUILD_TAG:latest"
  docker ps
fi


