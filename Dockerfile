# ╔════════════════════════════════════════════════════════════════════════════╗
# ║                                                                            ║
# ║                 __ \             |               _|_) |                    ║
# ║                 |   |  _ \   __| |  /  _ \  __| |   | |  _ \               ║
# ║                 |   | (   | (      <   __/ |    __| | |  __/               ║
# ║                ____/ \___/ \___|_|\_\\___|_|   _|  _|_|\___|               ║
# ║                                                                            ║
# ║           * github.com/paperwork * twitter.com/paperworkcloud *            ║
# ║                                                                            ║
# ╚════════════════════════════════════════════════════════════════════════════╝
FROM alpine:latest

RUN sed -i -e 's/v[[:digit:]]\.[[:digit:]]/edge/g' /etc/apk/repositories \
 && apk upgrade --update-cache --available \
 && apk add rust cargo openssl openssl-dev

WORKDIR /src
ADD . .

RUN cargo build --release \
 && mv target/release/gatekeeper /gatekeeper

CMD ["/gatekeeper"]
