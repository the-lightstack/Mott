# Language Specifications for **Motts**

Motts has (almost) no static keywords like other languages and is a bit similar to RockStar in the ability
to form actual sentences. (maybe check each word in a dict, so that it is an actual, english word >:) )

# What does a programming language need:
- Control flow (branching) [maybe impl with big/small start letter (big = forwards, small = backwards)]
- Variables (We will have two types of vars: numbers and strings, bools are just 1/0 and chars just a string)
- input/output
- Lines end with "." a full stop/point

## Variables
define a variable by simply having a word with 4 letters (first letter: uppercase = number, lowercase = string) followed by a spelled out number (listing digits (for now)) for numbers or the string for strings

Examples:
```mt
// Numbers
john Zero.
greg Minus one seven.
ella Two comma four seven.

// Strings
teas Hello there, what is up.
back 4.
```

## Changing vars

Adding - 2 chars / uppercase
Subtracting - 2 char / lowercase

Multiplying - 3 chars / uppercase
Dividing - 3 chars / lowercase

*Syntax always:* operation <op1> <op2> <destination>

## I/O
**Output**: just one uppercase letter followed by all the vars you want to print (until point)
**Input**: just one lowercase letter followed by a uppercase word if it is a number or a lowercase word if it is a String followed by the var you want to put it into. var has to exist.

## Control flow
labels are just words with exactly 6 letters
word that starts with either {S (smaller), E (equal), B (bigger), A (always)} followed by 2 vars an the name of the label (backwards) to jump to.

Example - Looping until number is equal to 4 and printing numbers

```mt
mott Zero.
four Four.
ones One.

letters. // Label: 1

D mott. // print mott

To mott one mott. // mott += 1
Scren mott four letters. // if ( mott < 4): goto letters;

done program is done.
T done.
```

# TL;DR
i/o - 1 letter
add/sub - 2 letters
mul/div - 3 letters
var - 4 chars
branch - 5 letters
labels - 6 letters



# manual example of tokenization
" mott zero two  "
// trimmed
"mott zero two"
// finding out type of token w/ len of first word

