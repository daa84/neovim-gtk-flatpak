#!/bin/sh

flatpak-builder --repo nvim-gtk --force-clean ./build org.daa.NeovimGtk.json
flatpak build-bundle ./nvim-gtk nvim-gtk_x86_64.flatpak org.daa.NeovimGtk

