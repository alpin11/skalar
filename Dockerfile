FROM rust:1.68 as builder
WORKDIR /usr/src/image-scaling
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
WORKDIR /usr/src/image-scaling
RUN apt-get update && apt-get -y install openssl ca-certificates curl \
  && rm -rfv /var/lib/apt/lists/*

RUN update-ca-certificates
COPY --from=builder /usr/local/cargo/bin/image-scaling /usr/local/bin/image-scaling

RUN adduser --disabled-password --gecos '' image-scaling
USER image-scaling
ENV USER=image-scaling

CMD ["image-scaling"]