# Reference material

`command-registry.json` is the command registry recovered from the reference
build, read as data and never executed. The reference registers every command
through:

```
AddCommandInfo|<name>|<summary>|<aliases>|
```

474 entries, 63 distinct command names. `scripts/compare_with_reference.py`
checks this port against it.

The registry is a list of facts about the reference's interface, not its code.
Nothing was copied from the binary: the implementation here is independent.
