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
  - text scrolls out of view too quickly
  - line wrapping is less annoying than scrolling all the time
  - if you need the width in rare cases, fullscreen does fine
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

Behaviour:

- rotate on scratchpad:
  It rotates to the next window in the ring.
  This prevents having to undo the scratchpad first.
  Scratchpad unsummons so that you only have to set the structure of the rings in your brain.
  This makes moving around easily predictable.
  You don't need to rotate back into the scratchpad, as you can just summon it.
- rotate on fullscreen:
  Rotates through the ring but keeps fullscreen status.
  Again to prevent having to unfullscreen it first before moving, increasing speed.
  The fullscreen status is kept because if you set a window fullscreen, it probably means that
  the particular windows works better for you that way. Maybe it wasn't big enough.
  If it were to go back to regular on rotation, it means that when rotating back into it you
  would have to fullscreen it yet again.
- sink windows on spawn:
  This means every window is sunk into a ring upon creation.
  It includes all windows including popup windows that usually keep floating.
  - decreases clutter if a popup is spawned floating on top, a third window exists on screen
  - more space for the popup content: e.g. file pickers can use the space well
  - more focus on the popup: e.g. saving a file, you don't need to focus on other windows
  - if need be, you can rotate back and forth extremely quickly as the popup is inserted right next
    to the previous window

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

## Usage

This is very much a tool developed for personal use and an exercise in design.
Do not expect to be able to use this with little effort.

- keybindings designed for an obscure keyboard layout, you will need to change them
- only works for one or two screens, I think? (especially `move_focus` may misbehave)
- may need to be build against the develop branch of Penrose at any given moment


## License

Copyright (C) 2025 Cody Bloemhard

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
