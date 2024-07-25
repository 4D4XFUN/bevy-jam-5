# bevy-jam-5

Most of the files are copied from https://github.com/TheBevyFlock/bevy-template. I've omitted a lot to make it more lightweight.

Run the game on any system with `cargo run`

# Trunk
Install trunk with `cargo install --locked trunk`, then run the project in browser with `trunk serve --no-spa`

# Testing checklist

Here's an incomplete list of functions to try before committing your PR to check for breakages

- [ ] Camera scroll to zoom
- [ ] Camera overview button
- [ ] Running around and colliding with walls
- [ ] Rolling - known bug; https://github.com/4D4XFUN/bevy-jam-5/issues/43
- [ ] Enemies should chase player
- [ ] Respawn on death
- [ ] Ghosts spawn on respawn
- [ ] Music plays in background
- [ ] F1 to open world inspector
- [ ] F3 to draw dev overlays (grid, nearest walls, current grid square highlighted, etc.)
