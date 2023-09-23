# tgd - tribal government directory

A simple command line interface (cli) utility to query, search, and filter basic data about tribal governments. Information about the tribal governments was taken from the NCAI tribal government directory. 

# Install

```console
$ cargo install tgd
```

# Usage

Running `tgd --help` should provide a helpful overview of what the cli can do. Some examples:

## List the names of all tribal governments
```console
$ tgd list
```

## List the tribal governments websites that use .org domains
```console
$ tgd list --websites dot-org
```

## List the tribal governments websites that use .com domains
```console
$ tgd list --websites dot-com
```

## List tribal governements whose name includes search term
```console
$ tgd list --name Muscogee
```
