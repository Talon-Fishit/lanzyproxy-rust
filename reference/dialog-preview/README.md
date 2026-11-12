# Growtopia dialog preview

A browser page that renders Growtopia in-game dialogs from the markup the
Rust `lzdialog` crate produces, so the dialog builder can be checked by eye
without a Growtopia account.

Open `index.html` in a browser. The picker at the top chooses a dialog, the
left pane holds the raw markup and the right pane shows the result. The left
pane is editable: change the markup and the right pane re-renders.

## Running it

The page is one self-contained file with no dependencies and no build step.
`main.py` is only a convenience wrapper for taking a picture of it or driving
it for a self-test:

```sh
python3 main.py --screenshot page.png   # render the page to a PNG
python3 main.py --self-test report.json # drive the page and report what happened
```

## What it renders

The directive set the reference binary emits, recovered by static analysis
and listed in `../dialog-directives.json`. Every one of them has a visible
example in the `lanzy_extras` sample dialog.

Rows follow the game's look: labels are left aligned in a dark blue panel,
text boxes and text inputs are darker inset rows, small text is muted grey,
spacers and breaks are vertical gaps, margins add padding, checkboxes show a
ticked or empty box, buttons are green rounded pills, item pickers are chips,
quick exit is a right-aligned close link, and an icon renders as a box with
its name.

Text carries Growtopia colour codes: a backtick then one character, where
`2` is green, `4` is red, `9` is blue, `w` is white, `#` is purple, and a
double backtick resets.
