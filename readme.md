# QuikSearch

An attempt to create a fast QuickSilver-esque word matching algorithm.

## Abstract

Remember the famous Mac application "QuickSilver"? One of the massive advantages of it was "abbreviations", which allowed you to search for an app or file by typing a mnemonic for it. 

For example, if I had the following apps:

* Photos
* Photo Booth
* Adobe Photoshop
* Photo Magic

I could launch Photoshop by typing `phosh` (because **PHO**to**SH**op), Photo Booth by typing `phobo` (because **PHO**to **BO**oth), and so forth.

This is an attempt to create an algorithm which would be able to index a list of terms and provide a similar search ability with decent performance. 

As a "term" we shall use an arbitrary string consisting of "words" separated by whitespace. 
As a "query" we shall use an arbitrary string, case-insensitive, without any whitespace at all (because in QuickSilver spacebar was used to show more results).
The goal would be for the algorithm to return a list of the terms the user possibly wanted, ranked by similarity, keeping the original case and spelling exactly as when the term was learned by the database.

## Notice

I never studied any kinds of algorithms or data structures aside from failry shallow personal experience so feel free to suggest any changes and/or optimizations :P 

Consequentially I'm trying to not read anything like the Quiksilver source code or any related documentation to attempt and create the whole routine from scratch on my own.

## Theory

Currently this is basically a trie which contains term lists in the nodes. 

Strict matching works by following the trie according to the query and returning the term list, if any.

Prefix matching does the same but also recursively collects the terms from all other nodes X levels below the final found node as well.

Fuzzy matching works similar to prefix matching, however, when it reaches a mismatched node (when there is no match in current node's outputs with the current character) it tries to skip a FUZZ number of letters. Hence typing e.g. "phosh" will match against the term "Photoshop" when FUZZ ≥ 2 and DEPTH ≥ 1 (`[PHO]<- match, [to]<- fuzzy skip, [SHO]<- match, [p]<- depth`).

If the fuzzy algorithm still fails to match after skipping, it will try and do an unfuzzy match from the root starting at the current character. Hence typing e.g. "phobo" will match against the term "Photo Booth" (`[PHO]<- match [to]<- fuzzy skip [BO]<- restart new word from root [oth]<- depth`).

### Other thoughts

Currently a lot of false positives is cut off by adding a copy of the term with spaces removed into the trie. However this is not optimal. Probably it's better to keep track of the nodes somehow and when rematching a new word from the root, filter off those that don't have any relation to the already matched words. (Add nodes between words in addition to ones between letters, and do fuzzy matching over them rather than the root?)

Otherwise issues like this arise (second item should not be returned, but it is, due to `[UMB]` triggering a root rematch and matching with "Umbrella"):

```
> hereumb
You might have meant: [
    "Here Comes Mr. Umbrella",
    "WORLD\'S END UMBRELLA",
]
Search took 1.2266ms
```

P.S. Disabling root-rematching while keeping the spaceless copy thing, but not giving up on a fuzzy mismatch, seems to have improved the situation significantly with shorter items, however not with long ones (`hereumb` matches properly but `thocherblo` won't match "Thousand Cherry Blossoms" anymore).

## Example output

Example output as of the initial commit with FUZZ=5, DEPTH=30, which seems to work at least sometimes. (Lines prefixed with `>` are user input)

```
Predictive Search
Loading took 18.2265ms
Enter query, Ctl-C to exit.

> ghorul
You might have meant: [
    "Ghost Rule",
]
Search took 637.5µs

> systemlov
You might have meant: [
    "Systematic Love",
]
Search took 871.5µs

> lucdream
You might have meant: [
    "Lucid Dreaming",
]
Search took 1.0359ms

> senbon
You might have meant: [
    "Thousand Cherry Blossoms (Senbonzakura)",
]
Search took 801.7µs

> deepseaund
You might have meant: [
    "Deep Sea City Underground",
]
Search took 753.4µs

> pimoo
You might have meant: [
    "Pink Moon",
]
Search took 683.9µs

> colorse
You might have meant: [
    "Colorful × Sexy",
]
Search took 825.1µs

> solend
You might have meant: [
    "Solitude\'s End",
    "Solitude\'s End -extend edition-",
]
Search took 1.2576ms

> worlendan
You might have meant: [
    "World\'s End Dancehall -Live Dance Edition-",
]
Search took 872.8µs

> worlenum
You might have meant: [
    "WORLD\'S END UMBRELLA",
]
Search took 544.5µs

> hostr
You might have meant: [
    "Holy Star -DIVA mix-",
]
Search took 1.0205ms

> onrock
You might have meant: [
    "on the rocks",
]
Search took 953.4µs

> tilim
You might have meant: [
    "Time Limit",
]
Search took 582.1µs

> bleybr
You might have meant: [
    "Bless Your Breath",
]
Search took 808µs
```