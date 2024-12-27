FROM ubuntu

ENV LANG C.UTF-8
ENV LC_CTYPE C.UTF-8
ENV PYTHONIOENCODING utf-8

RUN \
  apt-get update && \
  apt-get install -y -q software-properties-common && \
  add-apt-repository -y ppa:kicad/kicad-8.0-releases && \
  apt-get update && \
  apt-get install -y -q \
    kicad \
  && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*
