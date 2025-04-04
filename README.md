# rmx

Command line tool to delete files based on their extension.

## Usage

> [!NOTE]
> `rmx` does not delete directories, nor socket or fifos or other special files. It will only deletes regular files.

### Basic arguments

File deletion is **not recursive by default** (can be set with `-r/--recurse`), and the matching files are not shown for improved performance (can be set with `-l/--list`).

Hidden files and directories are ignored by default (can be set with `-a/--all`).

By default, a confirmation prompt is shown before deleting files (can be removed with `-f/--force`).

Finally, to make a dry run, use `-n/--dry-run` (which will show the files that would be deleted in a normal run).

### Examples

```bash
rmx -h # or --help: Show help message
rmx -V # or --version: Show version

rmx yaml yml # Remove files in current directory with extension .yml and .yaml
rmx -i yml yaml # Remove files except the ones with extension .yml and .yaml

rmx -ri yml yaml # Recursively remove files in current directory except the ones with extension .yml and .yaml
```

### Presets usage

Check out `/etc/rmx/rmx.conf` for the default configuration file.

```bash
rmx --preset latex --config rmx.conf # Remove all latex-related files, specified in rmx.conf
```

## Installation

### Install from `.deb` package

This is the recommended way to install `rmx`, as it will automatically add the command to your `PATH`, add the man page, and create the file `/etc/rmx/rmx.conf`.

Download the `.deb` file from the latest release, and install it using:

```bash
sudo dpkg -i rmx_*.deb
```

### Install from source

Generating the man page (using `clap_mangen`):

```bash
cargo run --bin man --features mangen > rmx.1
gzip rmx.1
sudo mv rmx.1.gz /usr/share/man/man1/
```

Building the project:

```bash
cargo build --release
```

Then, make sure to add the binary to your `PATH`.

Finally, if you plan on using presets, or just want a complete installation, run the following command to copy the [default configuration file](rmx.conf) to `/etc/rmx/rmx.conf`:

```bash
sudo mkdir -p /etc/rmx
sudo cp rmx.conf /etc/rmx/
```

## Testing

To test the project (unit tests and integration tests), run:

```bash
cargo test
```
