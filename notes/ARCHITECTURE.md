# Architecture overview for the Retiscope project

This document provides a general overview of the reticulum project. It includes images for ease of understanding.

## The Retiscope Node
![Architecture overview for a retiscope node](./images/architecture_retiscope_node.svg)
A Retiscope Node is a special type of node that lives as a part of the reticulum network. Its primary purpose is to collect data about the structure of the network and to display it to the user in an easy to understand manner. The node itself is comprised of four key pillars: **The Frontend**, **The Database**, **The Configuration** and  **The Daemon**.

- The daemon is the primary interface and listener which listens to the reticulum network. It is configured using configured using special configuration files. The daemon should listen changes to these files and reload if necessary. It periodically sends relevant data to a database.

- The database contains all of the data collected by one or more daemons. Each database should be schemaful and only provide data in the correct format.

- The frontend is the primary user interface. It contains a node graph for network topography visualization. In the future it will also offer remote management capabilites.

- The configuration is a set of files which dictate the behaviour of both the frontend and the daemon. In the future the configuration may be changed using an inbuilt interface in the frontend.

The primary Retiscope node is depicted in white. The items with a green color represent both a regular reticulum node as well as an anchor node. The remaining area outlines how a possible remote management interface may look like.


## The Anchor Node
![Architecture overview for an anchor node](./images/architecture_anchor_node.svg)

Anchor nodes are special reticulum nodes that are part of the the retiscope specification. Anchor nodes exist to aggregate services and to reduce network noise.

In oder for a anchor node to function other nodes must have special 'register-service' destinations. These destinations then link to the anchor node and request to be added to the service registry. The service registry is an endpoint/destination that periodically announces its presence. Users may connect to an anchor node to request a list of available services and their metadata. An anchor node does NOT ensure that the service is reachable by the client.
The anchor node requires each service node to maintain a link to it. Should a link be dropped then said service must be marked as stale in the service registry. Should a service remain stale for a prolonged period of time it should then be removed from the service registry.

### Process
The service node requests to be added to an anchor node.

The anchor nodes accepts.

A link between the anchor node and the service is maintained.

A retiscope node now connects to the anchor node and requests the available services.

The anchor node responds.

The retiscope node now attempts to connect to the destinations.

The service responds and links to the retiscope node.
