FROM phusion/baseimage:0.11 as builder
LABEL maintainer="tailinzhang1993@gmail.com"
LABEL description="This is the build stage for Bandot. Here we create the binary."

ENV DEBIAN_FRONTEND=noninteractive

ARG PROFILE=release
WORKDIR /bandot

COPY . /bandot

RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
        export PATH=$PATH:$HOME/.cargo/bin && \
        scripts/init.sh && \
        cargo build --$PROFILE

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer="tailinzhang1993@gmail.com"
LABEL description="This is the 2nd stage: a very small image where we copy the Bandot binary."
ARG PROFILE=release
COPY --from=builder /bandot/target/$PROFILE/bandot-node /usr/local/bin

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /bandot bandot && \
	mkdir -p /bandot/.local/share/bandot && \
	chown -R bandot:bandot /bandot/.local && \
	ln -s /bandot/.local/share/bandot /data && \
	rm -rf /usr/bin /usr/sbin

USER bandot
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/bandot-node"]
