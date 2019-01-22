
# About

A small learning project created due to a need for a highly configurable
backup program and to learn Rust better.

# License
The project is licensed under the [MPL (Mozilla public license)](https://www.mozilla.org/en-US/MPL/2.0/).
The license is included in the LICENSE.txt

# Installation
Read the USAGE.md file.

# Main features

These features **might not yet be implemented** but are the main ones that
will be implemented. When all of these are complete the project will officially
reach version **1.0.0** and will be considered "stable".

- [x] backing up modified files
    - [x] multi-threaded (configurable)


- [x] multiple configurable backup targets
    - [x] each has an optional tag for identification (manual mode and logging)
    - [x] option to ignore files and folders (possibly regex)
    - [x] option to disable auto backup (manual mode)
    - [x] number of copies to keep (>1 disables timestamp checks)
    - [x] option to enable always copy


- [ ] restore from backup
	- [ ] restore backed up targets to their original location
	- [ ] option for selecting a new location
	- [ ] option to select which revision to restore


- [ ] daemon mode on Unix (pid-s and other runtime files are saved in a "run" folder)
    - [ ] intervals for backing up
    - [ ] possible windows service


- [x] manual mode (command line)
	- [x] manually select backup target
	- [ ] manually select target and destination folder
	- [ ] other possible arguments such as --daemonize


- [ ] snazzy UI, complete with:
    - [ ] icons
    - [x] ASCII art
    - [ ] colored
    - [ ] with cool text
    - [ ] fancy words
    - [ ] lots of logging



# Secondary features

Features that might be implemented later.

- NCURSES (Unix, maybe windows if the packages work) config editor
    - configurable styles


- Maybe a GUI front-end
	- maybe
	- just maybe


- _possible_ resource limiting


- more advanced scheduling
    - day of the week
    - hours
    - conditional backup (backup based on a configurable if statement)


- daemon file watcher

- notifications

# Contributing

Ask for a pull request if you have a nice idea or something.
ALL contribution is licensed under the same license above.
