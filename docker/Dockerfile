# -.-. --- .--. -.-- .-. .. --. .... - BASE SYSTEM -.-. --- .--. -.-- .-. .. --. .... -
FROM rust:alpine AS base_system

LABEL maintainer="control-owl@r-o0-t.wtf" \
      version="1.0" \
      description="Build QR2M project with MUSL"

ENV PROJECT_DIR="/home/QR2M"
WORKDIR $PROJECT_DIR


# -.-. --- .--. -.-- .-. .. --. .... - SYSTEM PACKAGES -.-. --- .--. -.-- .-. .. --. .... -
RUN echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/main" > /etc/apk/repositories && \
    echo "https://dl-cdn.alpinelinux.org/alpine/v3.22/community" >> /etc/apk/repositories && \
    apk update && \
    apk add --no-cache \
        bash build-base git cmake ninja meson bison pkgconf-dev musl-dev autoconf automake libtool \
        openssl-dev openssl-libs-static glib-dev cairo-dev harfbuzz-dev gettext-dev libepoxy-dev \
        libx11-dev libxext-dev glslang glslang-dev glslang-static shaderc-dev shaderc-static \
        libxkbcommon-dev libxkbcommon-static expat-dev expat-static freetype-dev fontconfig-dev \
        brotli-dev gawk texinfo libpng-dev libjpeg-turbo-dev zlib-dev bzip2-dev py3-pytest \
        pixman-dev graphite2-dev graphite2-static libxrandr-dev libxrender-dev pango-dev musl-locales \
        libxfixes-dev libxcursor-dev libxi-dev pcre2-dev fribidi-dev util-linux-dev tiff-dev \
        util-linux-static libxcb-dev gdk-pixbuf-dev librsvg-dev vulkan-loader-dev vulkan-tools \
        xz-dev flex-dev flex-libs libxdamage-dev libunwind-dev libxau-dev po4a rust-bindgen \
        xcb-util-dev graphene-dev gtk4.0-dev libadwaita-dev appstream-dev appstream-glib-dev \
        yaml-dev yaml-static zstd-dev zstd-static nghttp2-dev nghttp2-libs libxml2-dev libxml2-utils \
        c-ares-dev c-ares-utils ca-certificates libeconf-dev itstool gperf uutils-coreutils \
        curl-dev rustc-dev ngtcp2-dev ngtcp2-static libidn2-dev libidn2-static libffi-dev libxslt \
        py3-pip python3 tzdata dbus desktop-file-utils docbook-xml docbook-xsl shared-mime-info \
        gnu-libiconv-dev gnu-libiconv-libs gettext-libs libxdmcp-dev gcc g++


# -.-. --- .--. -.-- .-. .. --. .... - BUILD SCRIPTS -.-. --- .--. -.-- .-. .. --. .... -
COPY docker/compile-circus.sh $PROJECT_DIR/compile-circus.sh
COPY docker/static_library/ $PROJECT_DIR/
RUN chmod +x $PROJECT_DIR/*.sh


# -.-. --- .--. -.-- .-. .. --. .... - DEPENDENCIES -.-. --- .--. -.-- .-. .. --. .... -
#|                Library          | Dependant          | Status
#| ------------------------------- | ------------------ | ------------------------------
RUN $PROJECT_DIR/gettext.sh       #| glib,cairo,pango   | Compiles in 502s
RUN $PROJECT_DIR/pcre2.sh         #| glib,gtk4          | Compiles in 32s
RUN $PROJECT_DIR/zlib.sh          #| glib,cairo,libxml2 | Compiles in 3s
RUN $PROJECT_DIR/libffi.sh        #| glib,gtk4          | Compiles in 22s
RUN $PROJECT_DIR/glib.sh          #| cairo,harfbuzz     | Compiles in 68s
#------------------------------------------------------------------------glib 627s
RUN $PROJECT_DIR/libexpat.sh      #| cairo              | Compiles in 27s
RUN $PROJECT_DIR/brotli.sh        #| cairo              | Compiles in 28s
RUN $PROJECT_DIR/xorgproto.sh     #| cairo,libxau       | Compiles in 4s
RUN $PROJECT_DIR/libxau.sh        #| cairo              | Compiles in 21s
RUN $PROJECT_DIR/libxdmcp.sh      #| cairo              | Compiles in 27s
RUN $PROJECT_DIR/libxcb.sh        #| cairo,x11          | Compiles in 41s
RUN $PROJECT_DIR/libx11.sh        #| cairo              | Compiles in 67s
RUN $PROJECT_DIR/libxrender.sh    #| cairo              | Compiles in 26s
RUN $PROJECT_DIR/libxext.sh       #| cairo              | Compiles in 30s
RUN $PROJECT_DIR/fontconfig.sh    #| cairo,gtk4         | Compiles in 29s
RUN $PROJECT_DIR/libbz2.sh        #| cairo,harfbuzz     | Compiles in 6s
RUN $PROJECT_DIR/cairo.sh         #| librsvg,harfbuzz   | Compiles in 30s
#------------------------------------------------------------------------cairo 336s
RUN $PROJECT_DIR/nghttp2.sh       #| curl               | Compiles in 38s
RUN $PROJECT_DIR/curl.sh          #| appstream          | Compiles in 68s
RUN $PROJECT_DIR/openssl.sh       #| appstream          | Compiles in 320s
RUN $PROJECT_DIR/xz.sh            #| appstream          | Compiles in 41s
RUN $PROJECT_DIR/libeconf.sh      #| appstream          | Compiles in 12s
RUN $PROJECT_DIR/libunistring.sh  #| appstream          | Compiles in 354s
RUN $PROJECT_DIR/appstream.sh     #| QR2M               | Compiles in 29s
#------------------------------------------------------------------------appstream 872s
RUN $PROJECT_DIR/freetype.sh      #| harfbuzz,gtk4      | Compiles in 26s
RUN $PROJECT_DIR/harfbuzz.sh      #| gtk4,              | Compiles in 104s
RUN $PROJECT_DIR/graphene.sh      #| gtk4               | Compiles in 5s
RUN $PROJECT_DIR/libxml2.sh       #| gtk4               | Compiles in 43s
RUN $PROJECT_DIR/fribidi.sh       #| gtk4               | Compiles in 7s
RUN $PROJECT_DIR/libepoxy.sh      #| gtk4               | Compiles in 25s
RUN $PROJECT_DIR/libtiff.sh       #| gtk4               | Compiles in 122s
RUN $PROJECT_DIR/libjpeg-turbo.sh #| gtk4               | Compiles in 23s
RUN $PROJECT_DIR/libpng.sh        #| gtk4               | fail
RUN $PROJECT_DIR/pixman.sh        #| gtk4               | Compiles in 21s
RUN $PROJECT_DIR/gtk4.sh          #| QR2M               | Compiles in 232s
#------------------------------------------------------------------------gtk4 633s
RUN $PROJECT_DIR/gdk-pixbuf.sh    #| librsvg            | Compiles in 12s
RUN $PROJECT_DIR/pango.sh         #| librsvg            | Compiles in 17s
RUN $PROJECT_DIR/cargo-c.sh       #| librsvg            | Compiles in 645s
RUN $PROJECT_DIR/libadwaita.sh    #| QR2M               |
RUN $PROJECT_DIR/librsvg.sh       #| QR2M               |
#------------------------------------------------------------------------libadwaita

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
