FROM rustlang/rust:nightly as build
WORKDIR /usr/src/puzzles.crosswise/
COPY . .
RUN cargo build --release

# TODO: We want this running on alpine eventually. I managed to get it
# partially running on alpine with "RUN apk add libgcc gcompat", but the
# __res_init symbol was not found. According to
# https://github.com/gliderlabs/docker-alpine/blob/master/docs/caveats.md#incompatible-binaries
# and https://github.com/ibmdb/node-ibm_db/issues/217, this needs to be fixed
# in the binaries which use the offending symbol, i.e. my dependencies. Will
# continue to look into running on a more lightweight image, but ubuntu is fine
# for now.
FROM ubuntu
RUN apt-get update && apt-get install -y openssl
COPY --from=build /usr/src/puzzles.crosswise/target/release/puzzles_crosswise /bin
CMD ["/bin/puzzles_crosswise"]
