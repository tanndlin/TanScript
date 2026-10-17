FROM rust:1.97

RUN apt-get update \
    && apt-get install -y --no-install-recommends nasm gcc-mingw-w64-x86-64 wine wine64 \
    && rm -rf /var/lib/apt/lists/*

ENV WINEDEBUG=-all

RUN wineboot --init && wineserver --wait
