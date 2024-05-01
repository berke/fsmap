# fsmap

## Description

fsmap is a small tool that scans a Unix filesystem and creates a
serialized listing of the filesystem contents (the "map").

The content includes the directory structure, file sizes, one
timestamp (the newest of the creation, access and modification
times with a resolution of one minute) and symbolic links.

Serialization is done using serde in RMP format.

- Distinct filesystems
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

Use `ls EXPR` to list entries matching `EXPR`.  Type `help`
to get a list of commands.

Regular expressions are processed using the excellent [regex](
https://docs.rs/regex/1.10.4/regex/index.html#syntax) crate.

Examples:
  `ls mkv$` - Simple regexes do not need to be quoted
  `ls '\.(mkv|mp4)$'` - Single quotes are needed if certain characters are presentothers
  `ls usr/share` - Regular expression is matched against full path...
  `ls %name share` - ...unless `%name` is used
  `ls mkv$ & usr/share` - Expressions can be combined using `&` (and)...
  `ls '\.qcow2$' | %larger 1G` - ...using `|` (or)
  `ls lapack \ %name ^lib` - ...using `\` (difference)
  `ls %after 2014-03-06 & %before 2014-03-09 & reg.*mp4` - Date operators
  `ls '(?i:\.jpeg$)'` - Case insensitive
- `quit` - exit

You can ^C in the middle of a listing to get back to the prompt.

## Performance

The map files have no index of any kind (except for the per-device
inode maps); fsmap will just gobble up everything and hold it in
memory.  This can amount to many gigabytes.  Search performance is sufficient
for my present needs.

The indices are quite large, but can be significantly compressed in my
tests down to 1/6th the original size using `xz`, while other tools provide
about 2/3rds reduction.

## Plans

Built-in compression and decompression filters would be helpful.

Scanning performance can be improved significantly by serializing while
scanning, and possibly by ditching Serde.

A mmappable index would be great, as load times are significant.

In hindsight, I probably should have stored timestamps for directories.
Also, taking the newest of the three Unix timestamps wasn't that good
of an idea, as filesystems are usually mounted with `atime`.

Feel free to post suggestions on Github.

Paged output.

Adding variables to the command language is tempting, but it's also
hard to avoid producing yet another crappy programming language.

## Rant about Signals

To detect ^C I ended up writing the 67 line `sigint_detector.rs` module
but only after spending a good afternoon trying to get `signal_hook` to
work, only to realize in horror that it totaled more than 3000 lines...
I don't miss dealing with `malloc()` and `strlen()` and consorts, but
I do miss the nice Unix module of OCaml (which probably even works under
Windows for most things.)

## Compatibility

- Works on Linux.
- Probably works on other Unix systems.
- It probably doesn't work on other operating systems such as AmigaOS,
  IBM OS/2 or Microsoft Windows.

## License

Whatever.  I've thrown in Apache and MIT.

## Author

Berke DURAK <bd@exhrd.fr>

## Change history

- 0.3: Proper CLI with expression parser
- 0.2: Major rework, renamed from slurp.
