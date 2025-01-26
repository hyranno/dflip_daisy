FROM ubuntu

ENV LANG C.UTF-8
ENV LC_CTYPE C.UTF-8
ENV PYTHONIOENCODING utf-8

USER root
SHELL ["/bin/bash", "-c"]

RUN \
  apt-get update && \
  apt-get install -y -q \
    curl \
    software-properties-common \
    build-essential \
    python3-pip \
  && \
# Install kicad
  add-apt-repository -y ppa:kicad/kicad-8.0-releases && \
  apt-get update && \
  apt-get install -y -q \
    kicad \
  && \
  pip3 install kikit --break-system-packages && \
# Install rmk
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
  source $HOME/.cargo/env && \
  rustup target add thumbv6m-none-eabi && \
  rustup target add thumbv7em-none-eabihf && \
  cargo install rmkit flip-link cargo-make && \
  curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh && \
# clean up
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*
