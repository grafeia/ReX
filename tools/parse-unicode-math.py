#!/bin/python
from collections import defaultdict

symbols = []
header = """\
// DO NOT MODIFY!  
//
// This file is automatically generated by the ../tools/parse-unicode-math.py file using the
// unicode-math-table.tex file taken from `unicode-math` on CTAN. If you find a bug
// in this file, please modify ../tools/parse-unicode-math.py accordingly instead.
//
// FIXME: We should probably use the official unicode standards source.

#![allow(dead_code)]
use phf;
use parsenodes::AtomType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub code: u32,
    pub atom_type: AtomType,
}

pub trait IsSymbol {
    fn atom_type(&self) -> Option<AtomType>;
}

impl IsSymbol for char {
    fn atom_type(&self) -> Option<AtomType> {
        match *self {
            'a'...'z' | 'A'...'Z' | '1'...'9' => Some(AtomType::Ord),
            '+' | '-' | '*' => Some(AtomType::Bin),
            '=' | '<' | '>' => Some(AtomType::Rel),
            '(' | '['  => Some(AtomType::Open),
            ')' | ']'  => Some(AtomType::Close),
            '.' | ',' => Some(AtomType::Punct),
            _ => None,
        }
    }
}
"""

# Convert ../unicode-math-table.tex atomtype to our AtomType.
convert_type = {
    "mathalpha": "Alpha",
    "mathpunct": "Punctuation",
    "mathopen": "Open",
    "mathclose": "Close",
    "mathord": "Ordinal",
    "mathbin": "Binary",
    "mathrel": "Relelation",
    "mathop": "Operator",
    "mathfence": "Fence",
    "mathover": "Over",
    "mathunder": "Under",
    "mathaccent": "Accent",
    "mathaccentwide": "AccentWide",
    "mathbotaccent": "BotAccent",
    "mathbotaccentwide": "BotAccentWide",
}

# TeX -> Unicode template
template = '    "{}" => Symbol {{ code: {}, atom_type: AtomType::{} }}, // {}\n'

# Parse 'unicode-math-table.tex'.  Store relavent information in
# `symbols` as 4-tuples:
#     (TeX command, Unicode, AtomType, Description)
with open('unicode-math-table.tex', 'r') as f:
    for line in f:
        code = "0x" + line[20:25]
        cmd  = line[28:53].strip()

        cursor = 56
        while line[cursor] != '}': cursor += 1
        atom = line[56:cursor]

        cursor += 2  # Skip next `}{` sequence
        desc = line[cursor:-3]

        symbols.append((cmd, code, convert_type[atom], desc))

# Write '.../syc/symbols.rs'
with open('../src/symbols.rs', 'w') as f:
    f.write(header)
    # TODO: Codepoint -> AtomType implementation & conflicts

    # Write TeX Command -> Symbol hashmap.
    f.write("\npub static UNICODEMATH: phf::Map<&'static str, Symbol> = phf_map! {\n")
    for tpl in symbols:
        f.write(template.format(*tpl))
    f.write('};')

# This list is sorted, ordered by code point.
# Print the distribution of code points
# Starting from first, to last.
collisions = defaultdict(set)
previous   = [0,"0"]
output = ""

for symbol in symbols:
    code          = int(symbol[1], 0)
    previous_code = int(previous[1], 0)

    # We have found a collision
    if code == previous_code:
        collisions[code].update([symbol, previous])
    
    # No collision, so count how far we've gone and print out symbols
    else:
        output += '.'*(code - previous_code - 1) + '#'
        previous = symbol

# This output is being used for helping me determine how to group
# the unicode points for when creating a map from unicode -> atom type.
# The groups still need to be figured out.
#
# . means there is no atom-type and # means the is.
print(output)
print("\n")

# Use this to determine the collisions from unicode -> atom type.
print(collisions)

# Default values for symbols that have multiple TeX commands.
# Currently only accent's collide with Accent vs AccentWide.
# We will choose AccentWide by default, but I don't think it's
# likely people will manual input combining characters.
collision_defaults = {
    "0x020D0": "AccentWide",
    "0x020D1": "AccentWide",
    "0x00302": "AccentWide",
    "0x00303": "AccentWide",
    "0x020D7": "AccentWide",
}