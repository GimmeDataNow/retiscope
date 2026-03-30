# Pages
- [ ] Dashboard (maybe)
- [ ] Node Graph 
  - [ ] Inspect Node
  - [ ] Select 2 Nodes
    - [ ] Estimate Path
- [ ] Connections
  - [ ] Server status
  - [ ] Server information
    - [ ] Connection type, address, arguments (if local) (all of this is info without requesting info from the server)
    - [ ] General destination anchor for retreiving more destinations
    - [ ] Log view destination
    - [ ] Announce stream destination
    - [ ] Active Links destination
  - [ ] Server inspection
    - [ ] Announce stream
    - [ ] Log streaming
    - [ ] Active/known links
    - [ ] Trusted Identities
  - [ ] Register new servers (save to file)
- [ ] Settings

# About Anchors, Nodes and Services

This is the current plan:
Each reticulum server has exactly one anchor destination. This destinaion address is known and
is used to initiate a connection to each retiscope node.

The anchor node is the only destination that is part of retiscope which sends out announces.
Each of these announces will roughly follow the format below for broadcasting additional info
for connection purposes.

Format:
[RETISCOPE]|[PROTOCOL_VERSION]|[FLAGS]

Example:
RS|0.1.0|0b00000001

## Flags

There should be a variety of flags available.
- [ ] IS_SERVER (does this destination serve data) (almost always true)
- [ ] REQUIRES_AUTH (password auth)
- [ ] TRUSTED_ONLY (identity based auth)
- [ ] LOW_B_W (warning for slow network speeds)

## Anchor response

The anchor should then respond with something like this:

[
  {
    "service": "retiscope-logs",
    "destination": "29adc268f67955b8459efbdfde141ab7",
    "flags": 1 (same flags as before expressed as an u8)
  },
  {
    "service": "retiscope-announces",
    "destination": "29adc268f67955b8459efbdfde141ab7",
    "flags": 1 (same flags as before expressed as an u8)
  }
]

The client may then parse the data and proceed at will.

## Log service

The log service should allow incoming links and simply send the latest logs as plain text to
each of links. This data would be send similar to how a live query sends data.

## Announce service

The announce service should allow incoming links and simply send a compressed announce stream
to each of the links. It would roughly look as follows:

[RETISCOPE_NODE]|[ANNOUNCE]

with the [ANNOUCE] looking something like this:

{
	destination: 29adc268f67955b8459efbdfde141ab7,
	hops: 3,
	iface: '7c9fa136d4413fa6173637e883b6998d',
	timestamp: d'2026-03-29T12:52:29.773544362Z',
	transport: 7cbbe5ada62d88ee2d4dbe0c3cb1bceb
}

# implementation

## General Flow

There is a file which tells the gui to connect to certain remote interfaces. The gui should
spin up its own interface for dealing with connections.

There should be a total of 3 different tasks retiscope needs to tackle:
1. the GUI
2. the CLI
3. the DB

Each of these would probably have to run independantly for things to work smoothly.
The GUI should be the visual interface to the entire stack. It should connect to the DB.
The CLI should connect to several "endpoints" and aggregate data.
The data would then be stored in the DB.

## GUI configuration

The gui should check for a file named "remote_connections.toml" that file will contain all of
the servers the GUI should interface with.
It might be better for the CLI to handle that because then the CLI can just collect different
endpoints with ease and dump it neatly into the database.
The GUI would only have to listen to the database. The GUI should still be able to configure the
remote_connections.toml such that upon update the CLI refreshes with the new parameters.

# Future Goals

### Remote databases
In the future there should be the option to connect to a database via the reticulum network.
This would allow for the bypassing of firewalls and better auth. The issue with this is the
need for a proxy since databases don't natively work via the reticulum network stack.

### Turning it into a Network Operations Center (NOC)
This means turning the "plain" observer into a program that can interface with other devices
and retreive all manner of data and also control/configure the devices remotely.
It would also allow for the management of known/trusted identities. Subdevision into special roles
may be of interest.
A significant issue would then be security. Some kind of MFA would be needed. Maybe a secondary
identity might solve this.
Another issue would be that a node might get managed by two managers at the same time. Thus a
locking mechanism may be needed. This locking mechanism might need periodic timeouts due to
deadlocks being a possibility.

