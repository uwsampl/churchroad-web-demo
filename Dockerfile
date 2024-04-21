FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
  autoconf \
  bison \
  build-essential \
  ccache \
  clang \
  curl \
  flex \
  git \
  libc6 \
  libc6-dev \
  libtool \
  python3 \
  wget \
  xz-utils

# Download and build libffi-emscripten.
WORKDIR /root
RUN \
  # Begin by installing emsdk.
  mkdir emsdk \
  && wget -qO- https://github.com/emscripten-core/emsdk/archive/176dec4875db4d74c2038d7e8af3395be38d222c.tar.gz | tar xz -C emsdk --strip-components=1 \
  && cd emsdk \
  && ./emsdk install latest \
  && ./emsdk activate latest \
  && . "/root/emsdk/emsdk_env.sh" \
  && cd /root \
  && mkdir libffi-emscripten \
  && wget -qO- https://github.com/hoodmane/libffi-emscripten/archive/44cb3268c778b0894a37cddc4ee7c89189197f55.tar.gz | tar xz -C libffi-emscripten --strip-components=1 \
  && cd libffi-emscripten \
  && ./testsuite/emscripten/build.sh

RUN apt install -y pkg-config npm tclsh

WORKDIR /root
ADD churchroad churchroad

# Build yowasp-yosys. I don't love that we need to clone instead of downloading
# a release tarball, but the release tarballs don't include the submodules. I
# also tried using a submodule, but then it doesn't have the .git directory,
# which it seems to depend on.
WORKDIR /root
ARG MAKE_JOBS=1
RUN \
  git clone --recursive https://github.com/YoWASP/yosys yowasp-yosys \
  && cd yowasp-yosys \
  && git checkout ca630ead0ca01c06c389176a163966edeb07f151 \
  && git submodule update \
  && . "/root/emsdk/emsdk_env.sh" \
  # We need the Churchroad backend to be part of Yosys. I tried enabling plugins
  # with wasm, but it was more effort than it was worth given that we could also
  # just hack the backend into the Yosys source. The comented out lines are
  # artifacts of me trying to get plugins to work.
  #
  # && sed -i 's/ENABLE_PLUGINS := 0/ENABLE_PLUGINS := 1/' build.sh \
  # && sed -i 's$LINKFLAGS += -Wl,--strip-all$LINKFLAGS += -Wl,--strip-all,-L/root/libffi-emscripten/target/lib$' build.sh \
  #
  # Enable parallel build.
  && sed -i 's$make -C yosys-build -f ../yosys-src/Makefile PRETTY=0 CXX="ccache clang"$make -j'${MAKE_JOBS}' -C yosys-build -f ../yosys-src/Makefile PRETTY=0 CXX="ccache clang"$' build.sh \
  # Add Churchroad backend to Yosys.
  && mkdir yosys-src/backends/churchroad \
  && echo "OBJS += backends/churchroad/churchroad.o" >> yosys-src/backends/churchroad/Makefile.inc \
  && cp /root/churchroad/yosys-plugin/churchroad.cc yosys-src/backends/churchroad/churchroad.cc \
  && ./build.sh \
  && cd npmjs \
  && python3 prepare.py \
  && npm install . \
  && npm run all




