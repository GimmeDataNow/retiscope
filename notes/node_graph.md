the node graph needs serious work.

All interfaces should be spawned in a circle around the parent node.
Each interface should be spawned with a fixed angle between each other. 

All other nodes should determine which is the closest interface and spawn in the
corresponding area.

It may also be smart to determine how many routing nodes there are and to subdivide
the area of each of the interfaces further.
Should this be implemented then all other nodes should spawn near the corresponding routing node.

