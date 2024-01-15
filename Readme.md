# evil

A program for visualising simulated scattering events.

## Getting started

When starting `evil` you will first see the picture of a detector. To
show scattering events, use the menu entry

    File -> open

and select one of the files in the `examples` subdirectory. `evil` can
read (potentially compressed) files in the
[hepmc2](https://crates.io/crates/hepmc2) and
[LHEF](https://arxiv.org/abs/hep-ph/0609017) formats.

Each coloured line shows an outgoing simulated particle, with the
length of the line indicating the magnitude of its momentum.

### Transverse momentum and azimuthal angle plots

To learn more about an event, you can activate plots under the
`Windows` menu entry. You can zoom in and out with the mouse wheel,
drag to change the shown region, and double click to reset the
plot. You can also change the style in which a particle is shown by
clicking on the respective marker, usually a box, circle, or
star. Right click on the plot to export it.

### Jet clustering

Under

    Settings -> Jet clustering

you can select an algorithm and parameters for clustering collimated
strongly interacting particles into jets.

License: GPL-3.0-or-later
