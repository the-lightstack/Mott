# Documentation for mott

# Introduction
mott (from french "mot" = word) is (maybe) the first programming language, that 
doesn't limit your creativity to preset keywords! 
Get `if`, `let` and `var` out of here!
All tokens are defined by the words length and it's case.

# Operations
This chapter will briefly go over all the possible operation in `mott` (9 in total)
*Remember*: All tokens/lines must end in an period/dot/full stop (`.`)!

**Important**
1) In the below examples I use comments, but they are **not** yet supported by motts!
2) The `Arg Count` in the tables refers to arguments which do **not** include the token that defines the action.
   Therefore the action `P form hello.` would have **2** arguments: form and hello

Table of Contents
================
[Variables](#vars)
[Printing](#print)
[Input](#input)
[Addition](#add)
[Substraction](#sub)
[Multiplication](#mul)
[Division](#div)

----------------

## Var
| Key         | Value       |
|-------------|-------------|
| Case        | upper/lower |
| Word Length | 4           |
| Arg Count   | 1 - ∞       |

This is most likely the most common operation you will use, since there are **no string/number constants** - huray!
There are two types of Variables: Numbers (double) and Strings.
The word you use as an identifier for "Var" will become the variables name or identifier.
Let's see this in action:
```
mott Zero. // eg. mott = 0
dumb hello there, this is a string!.
pleb One three Three seven. // eg. pleb = 1337
case Minus seven comma three five. // case = -7,35
```
As you might have seen, you ~have to~ may finally spell out numbers.
**Important**
When declaring a **number*, the first argument has to be uppercase,
meanwhile *strings* are indicated by a lowercase first arg.
You can use the mot `Minus` to declare a negative number and `comma` to declare the numbers after 
the - you guessed it - comma!

## Print
| Key         | Value       |
|-------------|-------------|
| Case        | upper       |
| Word Length | 1           |
| Arg Count   | 1 - ∞       |

A programming language would not be a programming language withoug some I/O so lets get to work on that.
Printing to the console is really simple: Use an uppercase letter followed by all the vars you want to print.
```
form some string here. // declaring a string variable
numb Five zero two comma three.
D form.
B numb.
P form numb.
```
Another quick side note:
Since it is impossible to get a space, newline or dot into a string, predefined vars exist for them.
`newl` = "\n"
`spce` = " "
`dott` = "."

## Input
| Key         | Value       |
|-------------|-------------|
| Case        | lower       |
| Word Length | 1           |
| Arg Count   | 2           |

Input reads data from the user into the program - yay, interactivity!
Let's start with an example:
```
r NumPls result. // Reads a number into "result"
k Bummmm numm.   // Does the same, but shows how much creativity you have :)

l strInn vrbl.  // Reads a string into "vrbl" (reads until newline, not including it)
```
If you have the pattern-recognition skills of a Neural Network you might have already figured out the syntax:
<lowercaseLetter> <Uppercase-Word for Number/Lowercase for String> <destination>.

To see this in an actual (!) program, check out [this](./examples/add_1.mt) example.


## Add
| Key         | Value       |
|-------------|-------------|
| Case        | upper       |
| Word Length | 2           |
| Arg Count   | 3           |

All the following arithmethic operations follow the same, simple syntax:
<token> <op1> <op2> <dst> => `dst = op1 +|-|*|/ op2`
Example for 1 + 2:
```
onne One. // Define constant 1
twwo Two. // Define constant 2
rslt Zero. // Optional: predefine result var (would also be created by add!)

To onne twwo rslt. // Add 1 to 2 and place into rslt
P rslt.            // Print resulting value
```

## Sub
| Key         | Value       |
|-------------|-------------|
| Case        | lower       |
| Word Length | 2           |
| Arg Count   | 3           |

The same as add, just different token to indicate **Substraction**.
Example for 9 - 5
```
ninn Nine. // Define constant 9
blub Five. // Define constant 5
rslt Zero. // Optional: predefine result var (would also be created by add!)

bk ninn blub rslt. // Sub 5 from 9
P rslt.            // And print result
```

## Mul
| Key         | Value       |
|-------------|-------------|
| Case        | upper       |
| Word Length | 3           |
| Arg Count   | 3           |

The same as the other operations, just different token to indicate **Multiplication**.
Example for 2 * 4
```
twoo Two. // Define constant 2
more Four. // Define constant 4
           // showcasing, that you don't have to define the result var

Mut twoo more rslt. // Mul 2 * 4 => 8
P rslt.            // And print result
```

## Div
| Key         | Value       |
|-------------|-------------|
| Case        | lower       |
| Word Length | 3           |
| Arg Count   | 3           |

The same as the others, just different token to indicate **Division**.
Example for 10 divided by 5 (none-int results work too! e.g. 7/2 is no problem)
```
alot One zero. // Define constant 10
less Five.     // Define constant 5
           // showcasing, that you don't have to define the result var

bla alot less rslt. // Mul 10/5 => 2
P rslt.            // And print result
```

## Branching
| Key         | Value       |
|-------------|-------------|
| Case        | upper/lower |
| Word Length | 5           |
| Arg Count   | 3           |

A programming language would not work without branching - so here we go!
Branching just refers to interpreting a situation and reacting to it or - in simpler terms - conditional statements
like `if a == b {c()}`

Example time!
```
onne One.
numb Two.

Ebubl onne numb labels.
form they are not equal.
P form

labels.
```

Hmmm, what could `labels` be?
Maybe a [LABEL](#labels) ?!

But now back to Branches:
There are 3 different conditions:
[E]qual
[L]ess
[G]reater

The case does not matter.
```
Ecran a b destin. // jump to "destin" if a == b
lessr a b destin. // jump  to "destin" if a < b
Gregr a b destin. // jump  to "destin" if a > b

```

## Labels
| Key         | Value       |
|-------------|-------------|
| Case        | upper/lower |
| Word Length | 6           |
| Arg Count   | 0           |

Yes, after this you are done!
So a quick explanation of what labels are: They mark the location, a branch jumps to if the 
condition is fulfilled.
See the example in the [BRANCHES](#branching) category.

# Final Notes
1. Go add `motts` to the programming languages you have mastered in your resume.
2. Try to create something in motts and don't forget, that it is possible to write full sentences while writing code!
4. Examples can be found in [/examples](./examples) (if you want to add one, create a PR)
3. Have a good day :) 