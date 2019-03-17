FROM alpine:latest

RUN sed -i -e 's/v[[:digit:]]\.[[:digit:]]/edge/g' /etc/apk/repositories \
 && apk upgrade --update-cache --available \
 && apk add rust cargo openssl openssl-dev

WORKDIR /src
ADD . .

RUN cargo build --release \
 && mv target/release/gatekeeper /gatekeeper

CMD ["/gatekeeper"]
