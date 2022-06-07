#!/usr/bin/env fish
for p in kmacros_shim kproc_macros .; cargo publish --manifest-path $p/Cargo.toml; end
