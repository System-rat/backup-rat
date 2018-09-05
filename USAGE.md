# Usage
This document explains the usage of the backup-rat binary

# Installation
Requires a standard Rust installation (If you dont have it get it [here](https://rustup.rs/))
## Using crates.io
    $ cargo install backup_rat

## Using git
Clone the project 

    $ git clone "https://github.com/System-rat/backup-rat.git"
Enter the directory

    $ cd backup-rat
Install from the source

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
or `%HOME%/AppData/Roaming/System.rat/backup-rat/config.toml` for Windows, 
`$HOME/Library/Preferences/com.System.rat.backup-rat/config.toml` for OSX and uses the TOML
syntax (basically .ini files)

## The config file structure
All of the **Optional** variables are set to defaults in this example

**NYI** - Not yet implemented (don't use it, there is no need)

```toml

# Wether to use multi threading for copying or not
# Added: 0.2.0
multi_threaded = true 

# The number of threads to use in multi threaded copying (defaults to the number of cores)
# Added: 0.2.0
threads = 2
# NYI
daemon_interval = 0 

# NYI
color = false 

# NYI
fancy_text = true 

# NYI
verbose = false 

# NYI
runtime_folder = "" 

# This is 1 target, to configure more just put more of these [[target]] tags followed by the target declaration
# Added: 0.1.0
[[target]] 

# Optional: the tag for logging and for backing up (must use if the target is optional)
# Added: 0.1.0
tag = "Config"

# Optional: overrides the global `multi_threaded` flag
# Added: 0.3.1
multi_threaded = false

# The path to the file or folder to backup
# Added: 0.1.0
path = "/etc" 

# The destination of the backup (in this example the /etc folder will be in /mnt/Backup/etc)
# Added: 0.1.0
target_path = "/mnt/Backup" 

# Optional: If set to true the target will NOT be backed up by the *all* target
# Added: 0.1.0
optional = false 

# Optional: If set to true the files will NOT be checked for modification
# Added: 0.1.0
always_copy = false 

# Optional: A list of files to be ignored during backup
# Prefix the string with a r# to use a regex pattern
# If he string is not prefixed with a r# the full filename is evaluated
# Added: 0.3.0
ignore_files = [
    'r#.*\.ba(c|k)',
    '.build'
] 

# Optional: Same as `ignore_files` except it evaluates the folder path relative
# to the base directory (in example, target: "/home/user/Documents" the sub-folder
# "Rust/backup_rat" gets evaluated as such)
# Added: 0.3.0
ignore_folders = [
    'r#temp$',
    'build'
] 

# Optional: How many copies of the target to keep (folder targets only).
# The name of the directory to be copied will be put in the backup directory
# followed by `keep_num` number of subdirectories with the date of the backup as
# the name.
# DateTime directories are only created if `keep_num` is greater than 1.
# If `keep_num` is greater than 1, always_copy is ignored as it will always create a new directory.
# Example directory location: /mnt/Backup/etc/2042-2-18 12:00:43/
# Added: 0.4.0
keep_num = 2

# Example of a second target that uses the same tag
[[target]] 
tag = "Config"
path = "/home/USERNAME/.config"
target_path = "/mnt/Backup/Config"

```
