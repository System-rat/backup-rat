# Usage
This document explains the usage of the backup-rat binary

# Installation
Requires a standard Rust installation (If you dont have it get it [here](https://rustup.rs/))
## Using crates.io
    $ cargo install backup-rat

## Using git
- Clone the project 
    $ git clone "https://github.com/System-rat/backup-rat.git"
- Enter the directory
    $ cd backup-rat
- Install from the source
    $ cargo install


# Binary
Currently the binary **REQUIRES** a config file to operate correctly
(or not if you don't want any targets but that kinda defeats the purpose).

The version number before the **$** symbol indicates the version this feature was added

Backs up all NON-optional targets in the config file

    0.1.0 - $ backup-rat all 


Backs up all targets with the "*Config*" tag

    0.1.0 - $ backup-rat Config


Thats it... (for now)

# Configuration
The config file is located at `$HOME/.config/backup-rat/config.toml` for *NIX systems
or `%HOME%/AppData/backup-rat/config.toml` for Windows (thanks Microsoft) and uses the TOML
syntax (basically .ini files)

## The config file structure
All of the **Optional** variables are set to defaults in this example

**NYI** - Not yet implemented (don't use it, there is no need)

```toml

# NYI
multi_threaded = true 

# NYI
threads = 4 

# NYI
daemon_interval = 0 

# NYI
color = false 

# NYI
fancy_text = true 

# NYI
verbose = false 

# NYI
runfile_folder = "" 

# This is 1 target, to configure more just put more of these [[target]] tags followed by the target declaration
# Added: 0.1.0
[[target]] 

# Optional: the tag for logging and for backing up (must use if the target is optional)
# Added: 0.1.0
tag = "Config"

# The path to the file or folder to backup
# Added: 0.1.0
path = "/env" 

# The destination of the backup
# Added: 0.1.0
target_path = "/mnt/Backup/Config" 

# Optional: If set to true the target will NOT be backed up by the *all* target
# Added: 0.1.0
optional = false 

# Optional: If set to true the files will NOT be checked for modification
# Added: 0.1.0
always_copy = false 

# NYI
ignore_files = [] 

# NYI
ignore_folders = [] 

# NYI
keep_num = 1 

# Example of a second target that uses the same tag
[[target]] 
tag = "Config"
path = "/home/USERNAME/.config"
target_path = "/mnt/Backup/Config"

```
