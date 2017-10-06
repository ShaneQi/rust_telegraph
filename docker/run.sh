#!/bin/bash
docker run \
-it \
--name rusty_blog_generator \
-v /home/shane/rusty_blog:/rusty_blog \
-w /rusty_blog/generator \
rust:1.20 \
/bin/bash -c \
"\
apt update; \
apt install cmake -y; \
cargo run --release \
/rusty_blog/content \
/rusty_blog\
"
