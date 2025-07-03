FROM rust:alpine

# Install sys deps
RUN echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/main" > /etc/apk/repositories && \
    echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/community" >> /etc/apk/repositories && \
    apk update && \
    apk add --no-cache \
      bash curl build-base git cmake ninja meson bison \
      pkgconf-dev musl-dev \
      gettext-dev gettext-static \
      openssl-dev openssl-libs-static \
      glib-dev glib-static \
      cairo-dev cairo-static \
      harfbuzz-dev harfbuzz-static \
      libxml2-dev libxml2-static \
      libx11-dev libxrandr-dev libxrender-dev libxext-dev \
      libxfixes-dev libxcursor-dev libxi-dev \
      glslang glslang-dev glslang-static \
      shaderc-dev shaderc-static \
      libxkbcommon-dev libxkbcommon-static \
      zlib-static \
      pango-dev \
      fontconfig-dev \
      gdk-pixbuf-dev \
      librsvg-dev \
      vulkan-loader-dev vulkan-tools \
      xz-dev \
      flex-dev flex-libs \
      libxdamage-dev

ENV APP_NAME="QR2M"
ENV BUILD_PATH="docker/target/x86_64-unknown-linux-musl"
ENV OUTPUT_DIR="docker/target/x86_64-unknown-linux-musl/release"
ENV FEATURES="dev"
ENV TARGET="x86_64-unknown-linux-musl"
ENV OUTPUT="true"

# Install Rust MUSL target
RUN rustup target add x86_64-unknown-linux-musl

# Create non-root user
RUN addgroup -S app && adduser -S app -G app
USER app

WORKDIR /usr/src/app

# Copy and run build script
COPY --chown=app:app docker/compile-circus.sh /usr/src/app/compile-circus.sh
RUN chmod +x /usr/src/app/compile-circus.sh && /usr/src/app/compile-circus.sh

# Default command for interactive sessions
CMD ["bash"]