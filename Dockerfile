FROM scratch

WORKDIR /app

COPY --chmod=0755 ecscape ./ecscape

CMD ["/app/ecscape"]
