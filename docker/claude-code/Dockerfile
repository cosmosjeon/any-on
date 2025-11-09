# syntax=docker/dockerfile:1.7
FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive
ARG NODE_MAJOR=20
ARG CLAUDE_CODE_VERSION=2.0.31
ARG GH_CLI_VERSION=2.64.0

ENV ANYON_USER=anyon \
    ANYON_SECRET_DIR=/tmp/anyon-secrets \
    PATH="/home/anyon/.npm-global/bin:/opt/gh/bin:$PATH"

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
        gnupg \
        vim \
        xz-utils \
        unzip \
        build-essential \
        openssh-client \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js
RUN mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_MAJOR}.x nodistro main" > /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install GitHub CLI
RUN curl -fsSL https://github.com/cli/cli/releases/download/v${GH_CLI_VERSION}/gh_${GH_CLI_VERSION}_linux_amd64.tar.gz -o /tmp/gh.tar.gz \
    && mkdir -p /opt/gh \
    && tar -xzf /tmp/gh.tar.gz -C /opt/gh --strip-components=1 \
    && rm /tmp/gh.tar.gz

# Create non-root user and workspace
RUN useradd -m -s /bin/bash ${ANYON_USER} \
    && mkdir -p /workspace ${ANYON_SECRET_DIR} \
    && chown -R ${ANYON_USER}:${ANYON_USER} /workspace ${ANYON_SECRET_DIR}

USER ${ANYON_USER}
WORKDIR /workspace
ENV HOME=/home/${ANYON_USER}

# Use user-local npm prefix
RUN mkdir -p /home/${ANYON_USER}/.npm-global \
    && npm config set prefix /home/${ANYON_USER}/.npm-global

# Install CLI tooling required inside the container
RUN npm install -g @anthropic-ai/claude-code@${CLAUDE_CODE_VERSION} \
    && npm install -g @github/cli \
    && npm cache clean --force

CMD ["bash"]
