# Stage 1: Build enviroment
FROM rust:alpine AS builder
WORKDIR /usr/src/bday
RUN apk add --no-cache musl-dev
COPY Cargo.toml Cargo.lock .
COPY src/ ./src/
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN strip target/x86_64-unknown-linux-musl/release/bday

# Stage 2: runtime dependencies
FROM alpine:latest
WORKDIR /app-cache
# Get yt_dlp and dependencies
RUN apk add --no-cache \
  python3 \
  py3-pip \
  ffmpeg &&\
  pip install --break-system-packages yt_dlp boto3


# Add ShodOS identifiers to /etc/os-release
RUN  echo 'NAME="ShodOS"\nVERSION="1.0"\nID=shodos\nPRETTY_NAME="ShodOS 1.0"' > /etc/os-release 

# Copy python script to a dir in PATH and make executable
COPY --from=builder /usr/src/bday/target/x86_64-unknown-linux-musl/release/bday /usr/local/bin/
COPY create-r2.py /usr/local/bin/
COPY .env /usr/local/bin/
RUN chmod +x /usr/local/bin/create-r2.py
# Create 'user' to run commands for '/sudo'
RUN adduser -h /home/bday -D bday
WORKDIR /home/bday

ENTRYPOINT ["bday"]
