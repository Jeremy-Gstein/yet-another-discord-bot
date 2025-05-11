# Stage 1: Build enviroment
FROM rust:slim-bookworm AS builder

WORKDIR /usr/src/bday
COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./create-r2.py .
COPY ./src/ ./src/
RUN cargo build --release && \
    strip target/release/bday


# Stage 2: runtime dependencies
FROM rust:slim-bookworm

# Get yt_dlp and dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    python3 \
    python3-pip \
    ffmpeg && \ 
    pip install --break-system-packages yt_dlp boto3 && \
    apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

# Add ShodOS identifiers to /etc/os-release
RUN  echo 'NAME="ShodOS"\nVERSION="1.0"\nID=shodos\nPRETTY_NAME="ShodOS 1.0"' > /etc/os-release 

# Copy python script to a dir in PATH and make executable
COPY --from=builder --chmod=755 /usr/src/bday/create-r2.py /usr/local/bin/
COPY --from=builder /usr/src/bday/target/release/bday /usr/local/bin/
COPY .env /usr/local/bin/

# Create 'user' to run commands for '/sudo'
RUN useradd -m -d /home/bday -s /bin/bash bday

ENTRYPOINT ["bday"]
