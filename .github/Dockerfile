FROM debian:12-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY binaries/Linux-GNU-${TARGETARCH}-scrolls /bin/scrolls
RUN chmod +x /bin/scrolls
RUN ln -s /bin/scrolls /scrolls

ENTRYPOINT ["scrolls"]