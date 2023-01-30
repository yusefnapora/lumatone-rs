# App routes

Fair warning, this design doc will go out of date at some point. Ideally I'll circle back around to document things once they settle down and are implemented / sanity checked a bit.

## `/keymaps` - Keymap library / index view

Searchable list of Keymap preview cards.

Preview card includes:
- icon (e.g. pitch wheel)
- name
- If it's a harmonic / generated keymap:
  - Tuning name (e.g. 12TET, 31EDO, etc)
  - Scale name

Default action on tapping a card is to nav to the detail view, but long tapping could open a context menu. Context menu could include:
- send to device
- add to setlist
- add to favorites (default, built in setlist)

## `/keymaps/[id]/` - Keymap detail view

Expanded view of a given keymap.

Shows full keyboard view with colors applied and note labels (optional, enabled by default).


