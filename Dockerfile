FROM rust:1.67 as builder
WORKDIR /usr/src/image-scaling
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install openssl ca-certificates
RUN update-ca-certificates
COPY --from=builder /usr/local/cargo/bin/image-scaling /usr/local/bin/image-scaling
CMD ["image-scaling"]