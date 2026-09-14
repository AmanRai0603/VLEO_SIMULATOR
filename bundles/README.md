# Bundle recipes

Recipes live here; **bundle bytes do not**. The registry holds the bytes, and a
file-size limit on push keeps it that way.

The one exception is the shipped set — coefficients and climatology small
enough to travel inside the installer, so a fresh install runs before any sync.
`solar-drivers` is one of them.

`solar-weather` is the second, and for a different reason. It is 3.9 MB of CSV
carrying the whole record the solar-weather subsystem is evidenced against:
twenty-nine years of daily observation, every issued forecast, every alert, and
the derived monthly, cycle and regime tables. It is here rather than in the
registry because it is *evidence*, and evidence behind a registry nobody can
reach is evidence nobody checks. A reviewer who wants to know whether a node's
number is right can open the file it came from in a spreadsheet.

## Publishing

    cargo run -p xtask -- bundle publish bundles/solar-drivers/2026.09.04

Publishing hashes every payload file in the order the manifest lists them and
writes the result into the manifest. Publishing twice from the same input gives
the same hash — which is the property that makes verification mean anything.

Publication is **irreversible by design**: a published version is never
modified, and a correction is a new version. A licensed bundle needs two
approvers, because a bundle published with a fault in it is wrong,
confidently, on every machine that synced.
