# Port-swapping feature

## Problem statements

Many times two edges are connected to neighbor ports of a given node and cross each other. That unwanted cross can be avoid if the connected ports swapped.

Examples:

<example_1>
    <svg_file>
        temp\trellis-batch-5\b23.annotated.svg
    <svg_file>
    <current>
        * edges `A1-->B1` and `A2-->B1` are cross each other and the ports are neighbors.
        * edges `B1-->C1` and `B3-->C1` are cross each other and the ports are neighbors.
        * edges `B3-->C2` and `B3-->C1` are cross each other and the ports are neighbors.
    </current>
    <expected>
        * edges `A1-->B1` and `A2-->B1` swap the ports on B1 node to avoid collision.
        * edges `B1-->C1` and `B3-->C1` swap the ports on C1 node to avoid collision
        * edges `B3-->C2` and `B3-->C1` swap the ports on B3 node to avoid collision
    </expected>
</example_1>

<example_2>
    <svg_file>
        temp\trellis-batch-5\b15.annotated.svg
    <svg_file>
    <current>
        * edges `Vet-->Animal` and `Animal-->Dog` are cross each other and the ports are neighbors.
    </current>
    <expected>
        * edges `Vet-->Animal` and `Animal-->Dog` swap the ports on Animal node to avoid collision.
    </expected>
</example_2>

<example_3>
    <svg_file>
        temp\trellis-batch-5\b22.annotated.svg
    <svg_file>
    <current>
        * edges `B-->Out` and `A-->Out` are cross each other and the ports are neighbors.
        * edges `Hub-->E` and `Hub-->F` are cross each other and the ports are neighbors.
    </current>
    <expected>
        * edges `B-->Out` and `A-->Out` swap the ports on Out node to avoid collision.
        * edges `Hub-->E` and `Hub-->F` swap the ports on Hub node to avoid collision.
    </expected>
</example_3>

## Instructions

Validate the idea of port swapping feature.

1. Validate the idea and tell me any better idea if there's any
2. Checkt whether it's detectable that which edge crosses a given edge
3. Suggest me the best solution
