# syntax=docker/dockerfile:1.7
FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive
ARG NODE_MAJOR=20

# Install base tooling and Node.js
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
        gnupg \
        vim \
        xz-utils \
        build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_MAJOR}.x nodistro main" > /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install Claude Code CLI globally
RUN npm install -g @anthropic-ai/claude-code@2.0.31 \
    && npm cache clean --force

WORKDIR /workspace

ENV PATH="/root/.npm-global/bin:$PATH"

CMD ["bash"]
