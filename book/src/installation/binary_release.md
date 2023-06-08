# Binary Release

_Scrolls_ can be run as a standalone executable. 

## Download, install amd test

1. Download one of the binary release files from the [latest releases](https://github.com/txpipe/scrolls/releases) page.

2. Use the following command to extract and install the _Scrolls_ executable to `/usr/local/bin`:

```sh
    tar xz -C /tmp <scrolls-binary-release.tar.gz> && mv /tmp/scrolls /usr/local/bin
```

3. Test installation by running the following command. It should show the _Scrolls_ command-line help message.

```sh
    scrolls
```
4. If the last step does not work, make sure `/usr/local/bin` is in your `$PATH` variable or provide the complete path to the scrolls binary.

```sh
# Check $PATH variable
echo $PATH

# Use complete path to binary
/path/to/scrolls
```

See the [usage](../usage/index.md) page for running _Scrolls_.