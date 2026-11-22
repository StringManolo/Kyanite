#!/usr/bin/env sh
gh workflow run build-image.yml -f kernel_flavor=lts -f image_size_mb=1024 -f expect=kyanite
