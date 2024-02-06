FROM rust:latest as build
COPY . /app
ADD ./DockerCargoConfig $HOME/.cargo/config
WORKDIR /app
# RUN apt-get update && apt-get install -y musl-tools
# RUN rustup target add x86_64-unknown-linux-musl
# RUN cargo build --target=x86_64-unknown-linux-musl --release //alpine:latest +  x86_64 这种模式会引出其他问题
RUN cargo build --release

# FROM centos:latest 200M 可正常工作
# FROM rust:alpine3.15 800M 可正常工作
# FROM gcr.io/distroless/cc 38.3MB 可正常工作
FROM centos:latest
WORKDIR /web
COPY --from=build /app/target/release/aii_server /web
COPY .env /web
CMD ["./aii_server"]
