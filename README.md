# dldir [UNRELEASED]

Quick and dirty tool to download directories over HTTP(S)

This tool has been made for my private use in mind, and shouldn't be used except
if you know what you're doing

Symlinks are not supported and permissions are not kept

## Usage

All the files will be downloaded into the current directory, so you'll probably
want to create a new empty directory and `cd` into it before downloading
anything

```sh
# Generate a `dldir.txt` file describing the current directory's structure
dldir
# Download a directory ('https://a.valid/url/dldir.txt' must exist)
dldir https://a.valid/url
# Delete everything in the current directory before downloading
dldir https://a.valid/url -c
# Exclude some regex pattern of files/sub-directories (e.g. 'target|git')
dldir -x 'pattern' [...]
```

> See: `dldir -h`

## License

This is public domain, do whatever you want with it
