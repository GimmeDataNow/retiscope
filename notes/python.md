Currently the Reticulum-rs implementation seems to be flawed. It always seems to deadlock on very
busy channels. This is always true with 3+ channels.

Now the crazy idea would be to call the reticulum python router from the rust backend. I would
enable maxiumum logging and try to log every relevant packet and then parse it out from the rust
side. This is extremely cumbersome.
