# Fumomaker 🦀🔥🪡🧵

**Blazingly** ⚡️🚀 **fast** 🦀🔥 open source embroidery program. For the intersection of people who are fed up with
shittiness of ink/stitch and who's husquarna license was revoked.
Simple 🦀💣️ and free 🚀🟪️🚀🟨️⚡️ generation of embroidery patterns.

# Keybindings

`Ctrl+S` save scene file

`Ctrl+E` export embroidery json file

`Ctrl+Alt+RightMouseBtn` delete vertex from selected shape

`Ctrl+Alt+LeftMouseBtn` add vertex to selected shape

`LeftMouseBtn` start drawing

`Shift+LeftMouseBtn` start drawing a gap if perimeter tool is selected

`Ctrl+LeftMouseBtn` move vertex of selected shape

`LeftMouseBtn` hold embroidery config control arrow tips

`RightMouseBtn` select an object

`Escape` cancel something

`Z` finish drawing progress

`E` set embroidery config editor mode

`N` set editor mode to drawing

`Shift+C` set editor mode to stitch viewing

`Escape` in stitch view mode -> go to embroidery config mode

`1` select area drawing tool

`2` select thin path drawing tool

`3` select thick path drawing tool

`4` select symmetry movement drawing tool

`P` bind selected shape as parent shape for reparenting op

`H` bind selected shape/movement as ghost creation op prerequisite

`G` attach selected shape to group

`O` put selected shape into its own group

`M` create a ghost object

`Alt+Up` select previous color

`Alt+Down` select next color

`Delete` delete selected (if ghost is selected, its source is deleted)

`Shift+Delete` delete selected (if ghost is selected, source won't be affected)

Controlling shown stitch progress position:

- `Left` -= 1
- `Ctrl+Left` -= 10
- `Alt+Left` -= 100
- `Right` += 1
- `Ctrl+Right` += 10
- `Alt+Right` += 100

`T` toggle if selected area is gap

`T` toggle if selected thick line has pointed tips

`B` remove last vertex of currently edited shape

`W,A,S,D` moving camera

Wheel up/down zooms the camera.

## vp3 support 😑️

Conversion to vp3 is not blazingly fast though 😒️🐍️👎️. You need pyembroidery python package for that
