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

Fuzzy matching works similar to prefix matching, however, when it reaches a mismatched node (when there is no match in current node's outputs with the current character) it tries to skip a FUZZ number of letters. Hence typing e.g. "phosh" will match against the term "Photoshop" when FUZZ ≥ 2 (`[PHO]<- match, [to]<- fuzzy skip, [SHO]<- match, [p]<- depth`).

If the fuzzy algorithm still fails to match after skipping, it will try and do an unfuzzy match from the root starting at the current character. Hence typing e.g. "phobo" will match against the term "Photo Booth" (`[PHO]<- match [to]<- fuzzy skip [BO]<- restart new word from root [oth]<- depth`).


## Example output

Example output as of the initial commit with FUZZ=5, DEPTH=30, which seems to work at least sometimes. (Lines prefixed with `>` are user input)

```
Predictive Search
Loading took 8.2578ms, loaded 299 terms: about 36208 terms per second
Enter query without whitespace, Ctl-C to exit.
> ghorul
Search took 95µs
You might have meant: [
    "Ghost Rule",
]
> syslo
Search took 174.7µs
You might have meant: [
    "Systematic Love",
]
> lucdrm
Search took 538.9µs
You might have meant: [
    "Lucid Dreaming",
]
> senbon
Search took 51.4µs
You might have meant: [
    "Thousand Cherry Blossoms (Senbonzakura)",
]
> thchebl
Search took 339.7µs
You might have meant: [
    "Let Me Lose Myself in the Black Note",
    "Thousand Cherry Blossoms (Senbonzakura)",
]
> deepunder
Search took 97.5µs
You might have meant: [
    "Deep Sea City Underground",
]
> pimoo
Search took 102.1µs
You might have meant: [
    "Pink Moon",
]
> colorse
Search took 161.9µs
You might have meant: [
    "Colorful × Sexy",
]
> colmel
Search took 89.1µs
You might have meant: [
    "Colorful × Melody",
]
> solend
Search took 80.5µs
You might have meant: [
    "Solitude\'s End -extend edition-",
    "Solitude\'s End",
]
> woendan
Search took 171.5µs
You might have meant: [
    "World\'s End Dancehall -Live Dance Edition-",
]
> woenum
Search took 109µs
You might have meant: [
    "WORLD\'S END UMBRELLA",
]
> onrock
Search took 64.3µs
You might have meant: [
    "on the rocks",
]
> tilim
Search took 128.3µs
You might have meant: [
    "Time Limit",
]
> blybr
Search took 169.5µs
You might have meant: [
    "Bless Your Breath",
]
> miminishiage
Search took 729µs
You might have meant: [
    "Miku Miku Ni Shite Ageru",
]
> miku ageru
Search took 391.7µs
You might have meant: [
    "Miku Miku Ni Shite Ageru",
]
>
```