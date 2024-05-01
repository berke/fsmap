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

Indexing works well, search functionality is OK, memory consumption
and load times are a bit high.

## Use cases

Suppose you never got around to setting up that fancy RAID NAS with
ZFS and indexing and snapshots.  Instead, your drawers contain half a
dozen hard disk drives with mostly static contents.

Where did you put that particular file from five years ago?

Yes you did run `find /mnt/my-twelfth-drive >index12`
but that gives you no date information, no size information,
and grepping that isn't very convenient.

This tool provides a solution.

You create a map for each drive and keep them on your computer, so
that you can search their contents without having to plug each drive
in.

Searches go also much faster than using e.g. find on a live
filesystem, and you can search by date, file size, and use
boolean operators.

## Usage

### Map creation

To create a map:

`fsmap collect --out filesystem.mpk /path/to/my/filesystem`

To restrict the map to the first filesystem encountered, add `--one-device`

### Listing

To dump the map (with an optional filtering expression `EXPR`):

`fsmap dump [--expr EXPR] filesystem.mpk`

## Interactive mode

To interactively examine the map, type:

`fsmap examine filesystem1.mpk filesystem2.mpk ...`

Use `ls EXPR` to list entries matching `EXPR`.  Type `help`
to get a list of other commands.

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
memory.  This can amount to many gigabytes.  Search performance is
sufficient for my present needs.

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

## Important third-party crates

This tool uses the following important crates:

- `regex` for regular expressions
- `serde` and `rmp_serde` for MPK serialization and deserialization
- `rustyline` for command-line parsing

## Rant about signals

To detect ^C I ended up writing the 67 line `sigint_detector.rs`
module but only after spending a good afternoon trying read through
`signal_hook` to understand why ^C wasn't working.  It wasn't
`signal_hook`'s fault but I realized in horror that it totaled more
than 3000 lines, just to catch a signal...  come on now, I'm not
saying there can't be a use case for gold-plated, cross-platform,
thread and data race safe signal handling but I just want to catch
a ctrl-C and be able to figure out what's happening so that I can
debug it without getting lost in a maze of two crates and thousands
of lines.  This is getting ridiculous.

I don't miss the old C days of dealing with `malloc()` and `strlen()`
and consorts, but I do miss the nice Unix module of OCaml (which
probably even works under Windows for most things.)

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
