#!/bin/bash
docker run \
-it \
--name rusty_blog_generator \
-v /home/shane/rusty_blog:/rusty_blog \
-w /rusty_blog/generator \
rust:1.20 \
/bin/bash -c \
"\
cargo run \
/rusty_blog \
/rusty_blog\
"
