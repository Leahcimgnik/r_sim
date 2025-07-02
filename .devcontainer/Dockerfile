# Use the official slim Python image as the base image
FROM rust:1.86.0-slim-bullseye
ENV PROJECT_NAME rust_simulation
ENV WORKDIR /workspaces/${PROJECT_NAME}/

# Set the workspace directory
WORKDIR ${WORKDIR}

# Update and install essential packages
RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    git \
    curl