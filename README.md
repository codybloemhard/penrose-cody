# ringwm

A tiling window manager based on rings and super swallowing.
Build on top of the Penrose tiling window manager library: <https://github.com/sminez/penrose>.

## Explanation

Window swallowing is a feature of some tiling window managers (TWM).
Usually when a new window is created they rearrange the screen to fit it in.
Window swallowing is when the new window takes the position and size of an existing one,
thus effectively replacing it.
The existing window is said to be swallowed by the new one.
Usually this is based on rules where you want some types of windows to swallow some others.

Super swallowing, as I call it, is when the window manager tries to swallow almost always.
In this case, the screen splits into at most two columns.
When a third+ window is opened on a screen, it will swallow one of the two that are currently on
display.
Even popup windows swallow.
Usually a TWM makes them float over the tiled windows.

Rings are just list of windows.
You scroll through them in either direction, looping at the end.
They are like a tabbed layout, except without the tabs drawn.
Each column is a ring of windows.

## Features

- up to 2 columns per screen
- add new window into ring, swallow previous
- scroll through ring
- unswallow on close
- swap columns
- swap ring elements
- link scratchpad
- unlink scratchpad, if scratchpad is selected leave it in its current ring
- summon scratchpad, swallow ring
- unsummon scratchpad, return to ring
- rotate on scratchpad: unsummon scratchpad
- fullscreen with support for transparent windows (remove all other windows from screen)
- rotate on fullscreen: rotates through ring but keeps fullscreen status
- move between columns on and across screens with a single navigation function
- sink windows on spawn, thus inserting into a ring and swallowing the spawner

todo:

- context dmenu:
  - pop focused window out of ring into reikai
  - insert from reikai into focused ring
  - focus window
  - swap ring out

might do:

- move floating windows outside of rings, remember place to go back
- make so fullscreen toggle doesn't sink window

known bugs:

- bug: mpv window isn't draggable, fullscreen video only goes fullscreen in window
- bug: on fullscreen bar stays on some windows (st)
- bug: sometimes the visuals l-r and logical l-r columns order gets messed up
  when slow spawning windows are spawned?
- ? bug: ring killed upon first firefox window bug: moved to reikai but why?

## Design decisions

### Experience

A few observations from seven years of TWM usage:

You can subdivide your screen in many pieces but it is not that useful:

- each window is way too small
- quickly opening a new terminal is a pain as it splits nicely sized windows and it is too small
  to do anything with, needing another operation (like fullscreen)
- vertically split windows (rows instead of columns) are kinda useless
  - text scrolls out of view way to quickly
  - line wrapping is way less annoying than scrolling all the time
  - what do I need all that width for? Display a picture of Yunocchi? I can just fullscreen her.
- two vertical windows per screen is perfect
  - not too much clutter
  - perfect width for programming (about 100 chars wide)
  - web pages usually fit really well in half a screen
    - less space makes them cramped
    - more space and margins start appearing thus wasting space

In i3 I could easily open more windows, but I almost never opened more than 2 windows 
per screen per workspace.

Looking at another screen isn't that fast:

- you are focused on a window on screen 0
- you take your eyes off screen 0 and onto screen 1 by moving head and eyes
- you orientate and focus your eyes to the right depth
- text starts parsing in your brain after a bit
- you read whatever
- turn your eyes back to screen 1
- focus eyes again
- texts starts parsing with a little delay again

Switching out a window from right underneath your eyes is way quicker.

- you can actually start reading and understanding at least twice as fast if not faster
- you don't need to move your neck, head and eyes
- you don't need to refocus your eyes

More screen surface area doesn't help:

- looking up is a pain in the ass (screens above screens)
- looking left and right is fine but the further out the more annoying

I have two screens, the right column of the first one being right in front of me.
The left column of the first one and the left of the second one are one column right or left.
The right column of the second screen is furthest away.
It usually just has a very lightly used terminal to pad out the screen so there are two columns.
I could not have it and not miss it at all.
Three columns are king.

But I do want more than three windows.

- workspaces help
- a scratchpad terminal helps

### Solution

Rings solve all the problems.

- opening a window doesn't change the layout most of the time
  - more stable
  - more calm experience
- there is only three configurations: no windows, one big one, two columns
  - simple
  - always the right window size, no small windows with odd shapes
  - never have to resize anything
  - only two window orders: no fuzzing around reordering stuff
- scratchpad terminals are a feature, but end up getting used less
  for quick tasks, you just open a new one quickly, do stuff and kill it again
- only use about 1.5 screens most of the time
  - windows swap out right in front of you
  - minimal body and eye movement
  - quickest context switching (for the human using it)

### Unnecessary features

- tabs drawn per column
  - you know where your stuff is
  - more space for actual content
  - simple and clutter free design keeps you concentrated
  - if you can't see it you will find it more quickly (think about that one!)
- resize columns: could be implemented
  - not that useful
  - all important things (terminal, browser) work perfectly in two equal columns
- multiple scratchpad windows
  - needed in regular TWM because splitting the screen more for that extra window results
    in an unusable configuration
  - rings are already windows on top of windows
  - one scratchpad is available and more is just not necessary, it doesn't add to the experience
- animations
  - waste of time: increases time to focus eyes on text and start parsing
  - waste of compute cycles: energy being burned for nothing
  - increase code volume and complexity: increase bugs and vulnerabilities
- config file
  - just edit the progam and compile it
  - don't change your config often: make a good one and set the muscle memory
