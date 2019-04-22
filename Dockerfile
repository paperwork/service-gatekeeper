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

ARG APP_NAME
ARG APP_VSN

ENV APP_NAME=${APP_NAME} \
    APP_VSN=${APP_VSN}

# Fix for Rust package depending on an older version of libgit2
RUN apk upgrade && apk add libgit2

RUN sed -i -e 's/v[[:digit:]]\.[[:digit:]]/edge/g' /etc/apk/repositories \
 # && apk upgrade --update-cache --available \
 && apk add rust cargo openssl openssl-dev

WORKDIR /src
ADD . .

RUN cargo build --release \
 && mv target/release/${APP_NAME} /gatekeeper

CMD ["/gatekeeper"]
