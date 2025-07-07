FROM rust:alpine AS base_system

LABEL maintainer="control-owl@r-o0-t.wtf"
LABEL version="1.0"
LABEL description="Build QR2M project with MUSL"

ENV PROJECT_DIR=/home/QR2M
WORKDIR $PROJECT_DIR

RUN echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/main" > /etc/apk/repositories
RUN echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/community" >> /etc/apk/repositories
RUN apk update
RUN apk add --no-cache \
  bash build-base git cmake ninja meson bison pkgconf-dev musl-dev autoconf automake libtool \
  gettext-dev gettext-static \
  openssl-dev openssl-libs-static \
  glib-dev glib-static \
  cairo-dev cairo-static \
  harfbuzz-dev harfbuzz-static \
  libxml2-dev libxml2-static \
  libx11-dev libx11-static \
  libxext-dev libxext-static\
  glslang glslang-dev glslang-static \
  shaderc-dev shaderc-static \
  libxkbcommon-dev libxkbcommon-static \
  expat-dev expat-static \
  freetype-dev freetype-static \
  fontconfig-dev fontconfig-static \
  brotli-dev brotli-static \
  libpng-dev libpng-static \
  libjpeg-turbo-dev libjpeg-turbo-static \
  zlib-dev zlib-static \
  bzip2-dev bzip2-static \
  pixman-dev pixman-static \
  graphite2-dev graphite2-static \
  glib-dev glib-static \
  zlib-static \
  libxrandr-dev \
  libxrender-dev \
  pango-dev \
  libxfixes-dev libxcursor-dev libxi-dev \
  pcre2-dev pcre2-static \
  fribidi-dev fribidi-static \
  util-linux-dev util-linux-static \
  libxcb-dev libxcb-static \
  fontconfig-dev \
  gdk-pixbuf-dev \
  librsvg-dev \
  vulkan-loader-dev vulkan-tools \
  xz-dev \
  flex-dev flex-libs \
  libxdamage-dev \
  libunwind-dev \
  libxau-dev \
  po4a \
  rust-bindgen \
  xcb-util-dev \
  graphene-dev \
  gtk4.0-dev libadwaita-dev \
  appstream-dev appstream-glib-dev \
  yaml-dev yaml-static gperf itstool \
  zstd-dev zstd-static \
  nghttp2-dev nghttp2-static nghttp2-libs \
  c-ares-dev c-ares-utils ca-certificates \
  libcurl curl-dev curl-static

COPY docker/static_library/curl.sh $PROJECT_DIR/curl.sh
COPY docker/static_library/appstream.sh $PROJECT_DIR/appstream.sh
COPY docker/static_library/gdk-pixbuf.sh $PROJECT_DIR/gdk-pixbuf.sh
COPY docker/static_library/graphene.sh $PROJECT_DIR/graphene.sh
COPY docker/static_library/pango.sh $PROJECT_DIR/pango.sh
COPY docker/static_library/libxau.sh $PROJECT_DIR/libxau.sh
COPY docker/static_library/xz.sh $PROJECT_DIR/xz.sh
COPY docker/static_library/librsvg.sh $PROJECT_DIR/librsvg.sh
COPY docker/static_library/gtk4.sh $PROJECT_DIR/gtk4.sh
COPY docker/static_library/libadwaita.sh $PROJECT_DIR/libadwaita.sh
COPY docker/compile-circus.sh $PROJECT_DIR/compile-circus.sh


RUN chmod +x $PROJECT_DIR/curl.sh && $PROJECT_DIR/curl.sh
RUN cargo install cargo-c --features=vendored-openssl

RUN chmod +x $PROJECT_DIR/appstream.sh && $PROJECT_DIR/appstream.sh
RUN chmod +x $PROJECT_DIR/gdk-pixbuf.sh && $PROJECT_DIR/gdk-pixbuf.sh
RUN chmod +x $PROJECT_DIR/graphene.sh && $PROJECT_DIR/graphene.sh
RUN chmod +x $PROJECT_DIR/pango.sh && $PROJECT_DIR/pango.sh
RUN chmod +x $PROJECT_DIR/libxau.sh && $PROJECT_DIR/libxau.sh
RUN chmod +x $PROJECT_DIR/xz.sh && $PROJECT_DIR/xz.sh
RUN chmod +x $PROJECT_DIR/librsvg.sh && $PROJECT_DIR/librsvg.sh
RUN chmod +x $PROJECT_DIR/gtk4.sh && $PROJECT_DIR/gtk4.sh
RUN chmod +x $PROJECT_DIR/libadwaita.sh && $PROJECT_DIR/libadwaita.sh

RUN chmod +x $PROJECT_DIR/compile-circus.sh && $PROJECT_DIR/compile-circus.sh

# FROM rust:alpine AS build_project
# 
# ENV PROJECT_DIR=/home/QR2M
# WORKDIR $PROJECT_DIR
# 
# RUN apk update && apk add --no-cache rust cargo pkgconf git coreutils bash \
#     libxml2-dev curl-dev zlib-dev glib-dev glib-static \
#     openssl-dev pango-dev libadwaita-dev
# 
# COPY --from=base_system $PROJECT_DIR/compile-circus/STATIC $PROJECT_DIR/compile-circus/STATIC
# COPY --from=base_system $PROJECT_DIR/compile-circus/LOG $PROJECT_DIR/compile-circus/LOG
# 
# RUN chmod +x $PROJECT_DIR/compile-circus.sh && $PROJECT_DIR/compile-circus.sh
