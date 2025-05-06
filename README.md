# penrose-cody

todo:

- context dmenu:
  - yeet into reikai (current)
  - insert from reikai (select)
  - swap ring out

- ? move floating windows outside of rings, remember place to go back

- bug: fullscreen toggle sinks window
- bug: navigate with one fullscreen, cannot go in one direction from non fs ws
- bug: ring killed upon first firefox window bug: moved to reikai but why?
- bug: mpv window isn't draggable, fullscreen video only goes fullscreen in window

// sgi_video_sync_scheduler_callback WARN ] Resetting the vblank schedulerX connection

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
