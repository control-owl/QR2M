# -.-. --- .--. -.-- .-. .. --. .... - BASE SYSTEM -.-. --- .--. -.-- .-. .. --. .... -
FROM rust:alpine AS base_system

LABEL maintainer="control-owl@r-o0-t.wtf"
LABEL version="1.0"
LABEL description="Build QR2M project with MUSL"

ENV PROJECT_DIR="/home/QR2M"
ENV PKG_CONFIG_PATH="$PROJECT_DIR/STATIC/lib/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:/usr/lib/x86_64-linux-musl/pkgconfig:/usr/local/lib/pkgconfig"
WORKDIR $PROJECT_DIR


# -.-. --- .--. -.-- .-. .. --. .... - SYSTEM PACKAGES -.-. --- .--. -.-- .-. .. --. .... -
RUN echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/main" > /etc/apk/repositories && \
    echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/community" >> /etc/apk/repositories && \
    apk update && \
    apk add --no-cache \
        bash build-base git cmake ninja meson bison pkgconf-dev musl-dev autoconf automake libtool \
        gettext-dev gettext-static openssl-dev openssl-libs-static glib-dev cairo-dev harfbuzz-dev \
        libx11-dev libx11-static libxext-dev libxext-static glslang glslang-dev glslang-static \
        shaderc-dev shaderc-static libxkbcommon-dev libxkbcommon-static expat-dev expat-static \
        freetype-dev freetype-static fontconfig-dev fontconfig-static brotli-dev brotli-static \
        libpng-dev libpng-static libjpeg-turbo-dev libjpeg-turbo-static zlib-dev zlib-static \
        bzip2-dev bzip2-static pixman-dev pixman-static graphite2-dev graphite2-static \
        libxrandr-dev libxrender-dev pango-dev libxfixes-dev libxcursor-dev libxi-dev \
        pcre2-dev pcre2-static fribidi-dev util-linux-dev util-linux-static \
        libxcb-dev libxcb-static gdk-pixbuf-dev librsvg-dev vulkan-loader-dev vulkan-tools \
        xz-dev flex-dev flex-libs libxdamage-dev libunwind-dev libxau-dev po4a rust-bindgen \
        xcb-util-dev graphene-dev gtk4.0-dev libadwaita-dev appstream-dev appstream-glib-dev \
        yaml-dev yaml-static zstd-dev zstd-static nghttp2-dev nghttp2-libs libxml2-dev \
        c-ares-dev c-ares-utils ca-certificates libeconf-dev itstool gperf \
        curl-dev rustc-dev ngtcp2-dev ngtcp2-static libidn2-dev libidn2-static


# -.-. --- .--. -.-- .-. .. --. .... - BUILD SCRIPTS -.-. --- .--. -.-- .-. .. --. .... -
COPY docker/compile-circus.sh $PROJECT_DIR/compile-circus.sh
COPY docker/static_library/ $PROJECT_DIR/
RUN chmod +x $PROJECT_DIR/*.sh


# -.-. --- .--. -.-- .-. .. --. .... - DEPENDENCIES -.-. --- .--. -.-- .-. .. --. .... -
#|                Library          | Dep       | Status   |
#| ------------------------------- | --------- | -------- |
RUN $PROJECT_DIR/libxau.sh        #| cairo     | Compiles |
RUN $PROJECT_DIR/nghttp2.sh       #| curl      | Compiles |
RUN $PROJECT_DIR/xz.sh            #| appstream | Compiles |
RUN $PROJECT_DIR/libeconf.sh      #| appstream | Compiles |
RUN $PROJECT_DIR/curl.sh          #| appstream | Compiles |
RUN $PROJECT_DIR/graphene.sh      #| gtk4      | Compiles |
RUN $PROJECT_DIR/libxml2.sh       #| gtk4      | Compiles |
RUN $PROJECT_DIR/harfbuzz.sh      #| gtk4      | Compiles |
RUN $PROJECT_DIR/fribidi.sh       #| gtk4      | Compiles |
RUN $PROJECT_DIR/libepoxy.sh      #| gtk4      | Compiles |
RUN $PROJECT_DIR/libtiff.sh       #| gtk4      | Compiles |
RUN $PROJECT_DIR/fontconfig.sh    #| gtk4      | Compiles |
RUN $PROJECT_DIR/freetype.sh      #| gtk4      | Compiles |
RUN $PROJECT_DIR/glibc.sh         #| gtk4      | Compiles |
RUN $PROJECT_DIR/libffi.sh        #| gtk4      | Compiles |
RUN $PROJECT_DIR/libjpeg-turbo.sh #| gtk4      | Compiles |
RUN $PROJECT_DIR/libpng.sh        #| gtk4      | Compiles |
RUN $PROJECT_DIR/pcre2.sh         #| gtk4      | Compiles |
RUN $PROJECT_DIR/pixman.sh        #| gtk4      | Compiles |
RUN $PROJECT_DIR/zlib.sh          #| gtk4      | Compiles |
RUN $PROJECT_DIR/cairo.sh         #| librsvg   | Compiles |
RUN $PROJECT_DIR/pango.sh         #| librsvg   | Compiles |
RUN $PROJECT_DIR/gdk-pixbuf.sh    #| librsvg   | Compiles |
RUN $PROJECT_DIR/glib.sh          #| librsvg   | Compiles |
RUN $PROJECT_DIR/cargo-c.sh       #| librsvg   | Compiles |
RUN $PROJECT_DIR/librsvg.sh       #| QR2M      | Compiles |
RUN $PROJECT_DIR/appstream.sh     #| QR2M      | Compiles |
RUN $PROJECT_DIR/gtk4.sh          #| QR2M      | FAIL     |
RUN $PROJECT_DIR/libadwaita.sh    #| QR2M      |          |


# -.-. --- .--. -.-- .-. .. --. .... - PROJECT COMPILE -.-. --- .--. -.-- .-. .. --. .... -
RUN chmod +x $PROJECT_DIR/compile-circus.sh && $PROJECT_DIR/compile-circus.sh


#
#
#RUN chmod +x $PROJECT_DIR/curl.sh && $PROJECT_DIR/curl.sh
#RUN chmod +x $PROJECT_DIR/appstream.sh && $PROJECT_DIR/appstream.sh
#
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
