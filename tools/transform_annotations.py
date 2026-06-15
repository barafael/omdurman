"""Transform annotations.ron to new Faction-with-data format.

Dervish entries:
  faction: Some(Dervish),  ->  faction: Some(Dervish(tribe: <TRIBE>)),
  brigade: None,  ->  removed

BritishEgyptian entries:
  faction: Some(BritishEgyptian),  ->  faction: Some(BritishEgyptian(brigade: <VALUE>)),
  brigade: <VALUE>,  ->  removed
"""
import re

TRIBE_MAP = {
    "Taiasha": "Taiasha",
    "KhalifaAbdullah": "Taiasha",
    "Sherif": "Danagla",
    "AliWadHelu": "Hadendowa",
    "SheikElDin": "Degheim",
    "Yakub": "Jaalin",
    "OsmanDigna": "Hadendowa",
    "Hadendowa": "Hadendowa",
    "Baggara": "Baggara",
    "Jehadia": "Jehadia",
    "Mulazmin": "Mulazmin",
    "Kehena": "Kehena",
    "Degheim": "Degheim",
    "Danagla": "Danagla",
    "UpperJaalin": "Jaalin",
    "LowerJaalin": "Jaalin",
    "HadendowaForts": "Hadendowa",
}

def transform_file(path):
    with open(path) as f:
        text = f.read()

    lines = text.splitlines(keepends=True)
    out = []
    current_section = None
    i = 0

    while i < len(lines):
        line = lines[i]

        # Track current section key
        m = re.match(r'^(\s+)(\w+):\s*\[', line)
        if m:
            current_section = m.group(2)

        # Detect start of a sprite annotation entry: ((col, row), (
        if re.match(r'\s+\(\(\d+,\s*\d+\),\s*\(', line):
            # Collect the entire entry (balanced parens up to )),
            entry_lines = [line]
            depth = line.count('(') - line.count(')')
            j = i + 1
            while j < len(lines) and depth > 0:
                cl = lines[j]
                entry_lines.append(cl)
                depth += cl.count('(') - cl.count(')')
                j += 1

            # Transform within the entry
            entry_text = ''.join(entry_lines)

            has_dervish = 'faction: Some(Dervish),' in entry_text
            has_be = 'faction: Some(BritishEgyptian),' in entry_text

            if has_dervish:
                tribe = TRIBE_MAP.get(current_section, "Baggara")
                entry_text = entry_text.replace(
                    'faction: Some(Dervish),',
                    f'faction: Some(Dervish(tribe: {tribe})),'
                )
                entry_text = re.sub(r'\s+brigade: None,\n', '\n', entry_text)
            elif has_be:
                bm = re.search(r'brigade:\s*(\S+),', entry_text)
                if bm:
                    val = bm.group(1)
                    entry_text = entry_text.replace(
                        'faction: Some(BritishEgyptian),',
                        f'faction: Some(BritishEgyptian(brigade: {val})),'
                    )
                    entry_text = re.sub(r'\s+brigade: \S+,\n', '\n', entry_text)

            out.append(entry_text)
            i = j
            continue

        out.append(line)
        i += 1

    with open(path, 'w') as f:
        f.writelines(out)

if __name__ == '__main__':
    import sys
    path = sys.argv[1] if len(sys.argv) > 1 else 'omdurman-app/assets/annotations.ron'
    transform_file(path)
    print(f"Transformed {path}")
