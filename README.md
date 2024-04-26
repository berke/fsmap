# fsmap

## Description

fsmap is a small tool that scans a Unix filesystem and creates a
serialized listing of the filesystem contents (the "map").

The content includes the directory structure, file sizes, one
timestamp (the newest of the creation, access and modification
times with a resolution of one minute) and symbolic links.

Serialization is done using serde in RMP format.

- Distinc filesystems
- Hard links
- Symbolic links

The listing can then be loaded into memory and examined or dumped.

File content digests are not included.

## Status

Indexing works well but the search functionality is primitive.

## Use cases

Suppose you have half a dozen removable external hard drives with
mostly static contents.  You create a map for each of them and keep
them on your computer, so that you can search their contents without
having to plug each drive.  Searches go also much faster than using
e.g. find on a live filesystem.

## Usage

### Map creation

To create a map:

`fsmap collect --path /path/to/my/filesystem --out filesystem.mpk`

To restrict the map to the first filesystem encountered, add `--one-device`

### Listing

To dump the map:

`fsmap dump --path /path/to/my/filesystem --in filesystem.mpk`

## Interactive mode

To interactively examine the map, type:

`fsmap examine filesystem1.mpk filesystem2.mpk ...`

- `find REGEX` - find all entries across all loaded maps matching the
given regular expression (which can't contain spaces until I implement
a lexer, but then who puts spaces in filenames? -- it's frowned upon.)
- `findi REGEX` - same, but case-insensitive
- `limit N` - limit the number of displayed entries to `N`
- `quit` - exit

You can ^C in the middle of a listing to get back to the prompt.

## Performance

The map files have no index of any kind (except for the per-device
inode maps); fsmap will just gobble up everything and hold it in
memory.  This can amount to many gigabytes.  Performance is sufficient
for my needs.

## Compatibility

- Works on Linux.
- Probably works on other Unix systems.
- It probably doesn't work on other operating systems such as AmigaOS,
  IBM OS/2 or Microsoft Windows.

## License

Whatever.

## Author

Berke DURAK <bd@exhrd.fr>

## Change history

- 0.2: Major rework, renamed from slurp.
