## What an Announce Actually Gives You

When an announce arrives, you know:
- Identity X exists.
- It was received via interface I.
- It arrived with hop count H.
- It came from some next-hop neighbor N (at the transport layer).

That is a packet observation.
It does not automatically imply that:
- Your routing table has committed to that path.
- The route is currently considered optimal.
- The route is still valid.
- You can immediately establish a link to X.

Announces propagate opportunistically. Routing decisions are separate.

## What an Inferred Route Represents:

An inferred route exists when:
> Your routing algorithm has selected a next-hop for identity X and stored it in its routing table.

This implies:
- A next-hop identity/interface
- A chosen hop count
- Path validity according to routing rules
- Possibly expiration timing

This is a **control-plane state**, not just a data-plane observation.
