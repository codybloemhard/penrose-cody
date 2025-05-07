# ringwm

Window manager based on rings and super swallowing.
Build on top of the Penrose tiling window manager library: <https://github.com/sminez/penrose>.

todo:

- context dmenu:
  - yeet into reikai (current)
  - insert from reikai (select)
  - swap ring out
- explain this stuff

- ? move floating windows outside of rings, remember place to go back

- bug: fullscreen toggle sinks window
- bug: mpv window isn't draggable, fullscreen video only goes fullscreen in window
- ? bug: ring killed upon first firefox window bug: moved to reikai but why?

features:

- 2 cols
- rings
- add new window into ring, swallow previous
- scroll through ring
- unswallow on close
- swap cols
- swap ring elements
- link scratchpad
- unlink scratchpad, if scratchpad is selected leave it in its current ring
- summon scratchpad, swallow window
- unsummon scratchpad, return to ring
- rotate on scratchpad, unsummon scratchpad
- fullscreen with support for transparent windows (remove all other windows)
- rotate on fullscreen, rotates through col keeps fullscreen status
- move between clients across screens with single navigation function
- sink windows on spawn, thus inserted in ring and swallowing spawner
