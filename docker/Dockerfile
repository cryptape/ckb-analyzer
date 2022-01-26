FROM ubuntu:20.04

RUN apt update \
    && apt install -y wget

RUN wget https://github.com/cryptape/ckb-analyzer/releases/download/v0.1.2/ckb-analyzer-linux-x86_64.tar.gz \
    && tar xzf ckb-analyzer-linux-x86_64.tar.gz \
    && mv target/release/ckb-analyzer /bin/ckb-analyzer \
    && chmod +x /bin/ckb-analyzer \
    && rm -f ckb-analyzer-linux-x86_64.tar.gz

ENV POSTGRES_HOST       127.0.0.1
ENV POSTGRES_PORT       5432
ENV POSTGRES_DB         postgres
ENV POSTGRES_USER       postgres
ENV POSTGRES_PASSWORD   postgres
