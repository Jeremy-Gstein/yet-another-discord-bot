#!/usr/bin/bash

BUILD_TAG="bday"
CONTAINER_NAME="bday_app"

build_dockerfile() {
  # used to explicitly bypass cache
  local no_cache=""
  if [ "$1" = "--no-cache" ]; then
    no_cache="--no-cache"
  fi
  # set current image as old if present
  if docker image inspect "$BUILD_TAG:lastest" &>/dev/null; then
    echo "Tagging current image as $BUILD_TAG:old"
    docker tag "$BUILD_TAG:latest" "$BUILD_TAG:old"
  fi

  # Build new image 
  echo "Building new image"
  if ! docker build -t "$BUILD_TAG:latest" $no_cache .; then
    echo "Build Failed"
    exit 1
  fi
}

run_docker() {
  docker run -itd --env-file .env --name "$CONTAINER_NAME" --rm "$BUILD_TAG:latest" 
}

check_docker() {
  if docker inspect "$CONTAINER_NAME" &>/dev/null; then
    echo "Swapping container with minimal downtime..."
    docker run -itd --env-file .env --name "tmp_$CONTAINER_NAME" --rm "$BUILD_TAG:latest"
    sleep 5 # short buffer for container to init/start.

    # Stop old container
    docker rm -f "$CONTAINER_NAME"
    sleep 5
    docker rename "tmp_$CONTAINER_NAME" "$CONTAINER_NAME"

    # Remove old image if tagged :old
    if docker image inspect "$BUILD_TAG:old" &>/dev/null; then
      echo "removing old image..."
      docker rmi "$BUILD_TAG:old"
    fi 
  else
    echo "Starting new container..."
    run_docker
  fi
}

help_menu() {
  cat <<EOF
build.sh for yet another discord bot (yadb | bday)
Usage: ./build.sh [OPTIONS]

Options:
  -b    Build and deploy container with minimal downtime
  -c    Clean build (remove all images/container and rebuild)  
  -rm   Similar to -c except when passed, adds --no-cache to docker build
  -h    Show this help menu
EOF
}



case "$1" in 
  -b)
    echo "Building and deploying..."
    build_dockerfile
    check_docker
    ;;
  -c) 
    echo "Starting clean build..."
    docker rm -f "$CONTAINER_NAME" 2>/dev/null || true
    docker rmi -f "$BUILD_TAG" "$BUILD_TAG:old" "$BUILD_TAG:latest" 2>/dev/null || true
    build_dockerfile
    check_docker
    ;;
  -rm)
    echo "Building with --no-cache... This may take longer."
    build_dockerfile "--no-cache"
    check_docker
    ;;
  -h)
    help_menu
    ;;
  *)
    echo "Error: Invalid option. Use -h for help."
    help_menu
    exit 1
    ;;
esac
