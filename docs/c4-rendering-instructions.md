# Latest C4 rendering instructions

The version 4 of the C4 modelling provides new look and feel.
Read the instructions and adapt to the trellis svg rendering in C4 Diagrams.

## Generic instructions

### Shapes

* The width of the elements are constant, but aligned to the grid system.
* The length of the text are impacting only the height of the node.
* The nodes are filled with background color (default: white)
* The stroke colors can be varying based on the purpose.

### Texts, captions, labels

* The text colors are the same as the stroke colors.
* The node caption is always bold.
* All texts can be wrapped
* No text can have more than 5 lines. If the text is longer, cut it and put ... automatically at the end of the text.
* All nodes have the same text order:
    1. Caption line:
        * Bold
        * size = default size + 2
        * text wrapped
    2. Container type: 
        * size = default size - 1
        * single lined
    3. Descriptions:
        * default size
        * text wrapped
* There is a 4 px gap between Caption - Container Type - Description.
* All texts are aligned to the center
* There are one line gap on above and below the entire text area

```
+---------------------------------+
|                                 |  
|        Caption line 1           |
|        Caption line 2           |
|   [Container: ContainerTypes]   |
|       Description line 1        |
|       Description line 2        |
|       Description line 3        |
|                                 |
+---------------------------------+

```

## Person

Example shape definition:

```svg

<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg" style="background: transparent; background-color: transparent; color-scheme: light dark;" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" width="458px" height="458px" viewBox="0 0 458 458"><defs/><g><g data-cell-id="0"><g data-cell-id="ztPbCkVsUMvpaBotz4ge-2"><g data-cell-id="ztPbCkVsUMvpaBotz4ge-4"><g><rect x="4" y="183" width="450" height="270" rx="89.1" ry="89.1" fill="#ffffff" stroke="#000000" stroke-width="8" pointer-events="all" style="fill: light-dark(#ffffff, var(--ge-dark-color, #121212)); stroke: rgb(0, 0, 0);"/></g></g><g data-cell-id="ztPbCkVsUMvpaBotz4ge-7"><g transform="translate(0.5,0.5)"><path d="M 94 449 L 94 303" fill="none" stroke="#000000" stroke-width="2" stroke-miterlimit="10" pointer-events="stroke" style="stroke: rgb(0, 0, 0);"/></g></g><g data-cell-id="ztPbCkVsUMvpaBotz4ge-8"><g transform="translate(0.5,0.5)"><path d="M 364 450 L 364 303" fill="none" stroke="#000000" stroke-width="2" stroke-miterlimit="10" pointer-events="stroke" style="stroke: rgb(0, 0, 0);"/></g></g><g data-cell-id="ztPbCkVsUMvpaBotz4ge-3"><g><ellipse cx="229" cy="103" rx="100" ry="100" fill="#ffffff" stroke="#000000" stroke-width="8" pointer-events="all" style="fill: light-dark(#ffffff, var(--ge-dark-color, #121212)); stroke: rgb(0, 0, 0);"/></g></g></g></g></g></svg>

```

### Instructions

* Use stroke color #08427b for person
* The top circle diameter is the half of the width of the bounding rectangle
* The top circle overlaps the top edge by 10 px
* Only the rounded rectangle can grow vertically according to the text.
* The two vertical lines moved to the bottom of the node.
