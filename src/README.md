Timewaster
===

Or: How I learned to stop worrying and treat CPU cycles as the enemy
---

Okay, so the background for this was: I wanted to clean some the greasy fingerprints off my laptop, and clearly some got in the touchpad.
Not great. So, I wanted something to peg the laptop's 4 cores for a bit whilst I let it try it.
I have videogames that should do the trick (hello factorio! You magnificient CPU-hungry beast), but I wanted to try some Rust multithreading and considered it an opportunity: how quickly could I knock out a program whose entire purpose was to compute, for no reason other than the joy of computing? How poetic.

Problem Description
---
This program attempts to find a string whose first *n* bytes of its SHA256 hash are equal to itself (*n* is set at compile time, but could be made adjustable). Then it will pipe the result to stdout, and quit.

Usage
---
Um, if you really want to use this project then go ahead and `git clone` it. Otherwise have fun reading the source code and maybe don't use soap on your laptop. 
