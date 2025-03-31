FROM alpine:latest

ENV RUST_BACKTRACE=full
# anyhow creates a backtrace for every error, which can be quite taxing, turning it off for now
ENV RUST_LIB_BACKTRACE=0
ENV RUST_LOG=info

WORKDIR /app
COPY --chmod=0755 ecscape ./ecscape

CMD ["sleep", "infinity"]
