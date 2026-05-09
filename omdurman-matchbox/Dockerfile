FROM rust:slim AS builder
RUN cargo install matchbox_server

FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/matchbox_server /usr/local/bin/
EXPOSE 3536
CMD ["matchbox_server"]
