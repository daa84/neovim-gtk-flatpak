# Flatpak packge for NeovimGtk
Currently only bundle is available you can find it at [releases](https://github.com/daa84/neovim-gtk-flatpak/releases) page.
After installation NeovimGtk can be found at application menu or can be run from command line `flatpak run org.daa.NeovimGtk`

[![Github All Releases](https://img.shields.io/github/downloads/daa84/neovim-gtk-flatpak/total.svg)]()


# NeovimGtk configuration
nvim and nvim-gtk configuration files (`init.vim` `ginit.vim`) stored at `~/.var/app/org.daa.NeovimGtk/config/nvim/`.
autoload configuration stored at `~/.var/app/org.daa.NeovimGtk/data/nvim/site/autoload`.
This package contains also git to allow autoinstall of vim plugins. So you can just put `plug.vim` at autoload directory and start using it.
