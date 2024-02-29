FROM rustlang/rust:nightly as builder
WORKDIR /usr/src/shorter
COPY . .
RUN cargo install --path .

FROM ubuntu:23.10 as release

COPY --from=builder /usr/local/cargo/bin/link_shorter /usr/local/bin/link_shorter

EXPOSE 8000

CMD ["link_shorter"]