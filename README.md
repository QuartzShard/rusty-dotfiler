# rusty-dotfiler
rusty-dotfiler is a tool that will read a `filemap.toml` and hardlink a list of `source_paths` to `install_paths`.
Its' intended use is to link dotfiles from a central directory (e.g a git repo to keep synced over multiple devices) to their destinations in the filesystem, so that updates to the source files will be automatically present where they're needed.
After populating `filemap.toml` with your configs and their paths, you can run `./rusty-dotfiler check` to see whether they're alredy hardlinked, and `./rusty-dotfiler install` to remove the defaults if they're present and hardlink your configs.


#### But why?

So I have an excuse to write something in rust, mainly. There's plenty dotfile-managers out there, and plenty that will likely do this job better and with more features. 
If you chose to use this one, please note that this is my first rust program and I can't vouch for its' quality, but I do intend to use it myself.