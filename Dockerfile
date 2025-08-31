# Dockerfile for testing MLFS Bootstrap
FROM ubuntu

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    wget \
    curl \
    git \
    pkg-config \
    libssl-dev \
    texinfo \
    gawk \
    bison \
    flex \
    m4 \
    python3 \
    python3-pip \
    file \
    tar \
    xz-utils \
    gzip \
    bzip2 \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Add required dependencies for MLFS
RUN apt-get update && apt-get install -y \
    binutils-dev \
    gcc-multilib \
    g++-multilib \
    libc6-dev-i386 \
    && rm -rf /var/lib/apt/lists/*

# Create LFS user and directories
RUN groupadd lfs && \
    useradd -s /bin/bash -g lfs -m -k /dev/null lfs && \
    mkdir -p /mnt/lfs/{tools,sources,build} && \
    chown -R lfs:lfs /mnt/lfs

# Set up LFS environment
USER lfs
WORKDIR /home/lfs

# Copy source code
COPY --chown=lfs:lfs . /home/lfs/mlfs-bootstrap/

# Build the project
WORKDIR /home/lfs/mlfs-bootstrap
RUN cargo build --release

# Set up environment variables
ENV LFS=/mnt/lfs
ENV LC_ALL=POSIX
ENV LFS_TGT=x86_64-lfs-linux-gnu
ENV PATH=/mnt/lfs/tools/bin:/usr/bin:/bin:${PATH}
ENV CONFIG_SITE=/mnt/lfs/tools/share/config.site
ENV MAKEFLAGS='-j4'

# Create entrypoint script
RUN echo '#!/bin/bash' > /home/lfs/entrypoint.sh && \
    echo 'set -e' >> /home/lfs/entrypoint.sh && \
    echo 'cd /home/lfs/mlfs-bootstrap' >> /home/lfs/entrypoint.sh && \
    echo 'echo "🐱 Starting MLFS Bootstrap in dev mode..."' >> /home/lfs/entrypoint.sh && \
    echo './target/release/mlfs-bootstrap --dev "$@"' >> /home/lfs/entrypoint.sh && \
    chmod +x /home/lfs/entrypoint.sh

# Default command
ENTRYPOINT ["/home/lfs/entrypoint.sh"]
CMD ["--init"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ./target/release/mlfs-bootstrap --dev --list || exit 1

# Labels for metadata
LABEL maintainer="Anonymous Catgirl <catgirl@mlfs.dev>"
LABEL description="MLFS Bootstrap - Multilib Linux From Scratch build environment"
LABEL version="0.1.0"