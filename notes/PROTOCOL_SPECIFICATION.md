# Retiscope Protocol Specification (v0.1.0)

This document defines the wire format for Retiscope telemetry and management data over the Reticulum Network Stack (RNS).

## Announce Format

All Retiscope-enabled nodes must include a specific app_data payload in their RNS Announces to be discoverable by the Retiscope Visualizer.

**Format**: **RS**|**VERSION**|**FLAGS**|**OTHER_DATA**

| Component  | Type         |	Description                                          |
| :--------- | :----------- | :--------------------------------------------------- |
| RS         | String       | Static header identifying the Retiscope protocol.    |
| VERSION    | SemVer       |	The protocol version (e.g., 0.1.0).                  |
| FLAGS      | Bitmask (u8) |	Node capabilities and status.                        |
| OTHER_DATA | Optional     | Context-specific data (e.g., Battery, Load, or Name).|


**Flags**:
| Bit        | Name             |	Description                                                              |
| :--------- | :--------------- | :----------------------------------------------------------------------- |
| 1          |	IS_ANCHOR       |	Is the node an anchor. If this is true then IS_SERVER must also be true. |
| 2          |	IS_SERVER       |	Node serves data.                                                        |
| 3          |	TRUSTED_ONLY    |	Connection requires Identity authentication.                             |
| 4          |	MFA_REQUIRED    |	MFA signing may be required to access all feature.                       |
| 5          |	REQUIRES_AUTH   |	Connection requires Password authentication.                             |
| 6          |	MANAGEABLE	    | Node accepts remote configuration commands.                              |
| 7          |	LOW_B_W	        | Node is on a low-bandwidth link (LoRa/HF).                               |

## Retiscope Specific Destinations

| Destination String         | Description                                    |
| :------------------------- | :--------------------------------------------- |
| `retiscope.network.anchor` | The primary "Discovery" endpoint for a node.   |
| `retiscope.network.manage` | Protected endpoint for RPC and configuration.  |
| `retiscope.network.logs`   | Stream-based destination for system telemetry. |

`retiscope.network.anchor` should be the only retiscope specific destination that sends out announces. All other destinations should be prefer to be silent by default.

## Status Codes

This will work as an alternative to existing http status codes as they are not fully compatible with the reticulum network architecture.
```rust
#[repr(u8)]
pub enum RetiscopeStatus {
    // Success
    Success = 0x00,
    DataFollows = 0x01,
    PARTIAL_CONTENT = 0x02,
    CACHED_RESPONSE = 0x03, // "I'm the middleman. Here's the last thing I heard." (used for anchors or aggregators)
    STALE_DATA = 0x04, // "I'm the source. I'm alive, but my sensors are stuck."

    // Async
    Processing = 0x20, // equivalent to HTTP 102 (Processing),
    QUEUED = 0x21,
    DEPENDENCY_WAIT = 0x22, // "waiting for dependencies to finish"
    UPGRADING = 0x23 // Flahsing new firmware, do not interrupt
    
    // Client Side
    BadRequest = 0x40,
    MfaRequired = 0x42, // The 2FA Challenge we discussed
    AccessDenied = 0x43,
    PAYLOAD_TOO_LARGE = 0x44,
    RATE_LIMITED = 0x45,
    IDEMPOTENCY_VIOLATION = 0x46, // "Stop sending duplicate commands!"


    // Node/Environment Side
    DbError = 0x60,
    StorageFull = 0x61,
    LOW_POWER_MODE = 0x62,
    HARDWARE_FAULT = 0x63, // "cant provide data because sensor died"
    STORAGE_FULL = 0x64,


    // Mesh Side (Unique to Retiscope)
    PathLost = 0x80,
    LinkCongested = 0x82, // "Too many managers, try again later"
}
```
