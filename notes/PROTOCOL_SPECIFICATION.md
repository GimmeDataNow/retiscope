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
    // --- 0x00: Success & Informational ---
    Success              = 0x00,
    DataFollows          = 0x01,
    PartialContent       = 0x02,
    PotentiallyDead      = 0x03,      // Anchor: "Link dropped recently, path might still exist"
    StaleData            = 0x04,      // Service: "I'm alive, but my sensor data is old"

    // --- 0x20: Async & Lifecycle ---
    Processing           = 0x20,       // Keep-alive: "I'm working on it"
    Queued               = 0x21,       // "You're in line behind other Managers"
    DependencyWait       = 0x22,       // "Waiting for another node to respond"
    Upgrading            = 0x23,       // "Flashing firmware, do not interrupt"
    MAINTENANCE_MODE
    MaintenanceMode      = 0x24        // "I am alive but I am in read only mode for manual servicing"
    Rebooting            = 0x25        // Rebooting, link will drop
    
    // --- 0x40: Client/Request Errors ---
    BadRequest           = 0x40,       // Malformed command
    NotFound             = 0x41,       // Resource/Command doesn't exist
    MfaRequired          = 0x42,       // Identity verified, but need 2FA signature
    AccessDenied         = 0x43,       // Identity lacks permissions
    PayloadTooLarge      = 0x44,       // Exceeds MTU or Node RAM
    RateLimited          = 0x45,       // "You are talking too fast for this radio link"
    IdempotencyViolation = 0x46,       // "Already processed this Command ID"
    UnsupportedEncoding  = 0x47,       // "I can't read this binary format"
    ChecksumMismatch     = 0x48,       // "The Resource transfer finished, but the SHA-256 doesn't match. Send it again"

    // --- 0x60: Node/Hardware Errors ---
    InternalError        = 0x60,       // General software crash/DB error
    LowPowerMode         = 0x61,       // "Battery too low to perform this action"
    HardwareFault        = 0x62,       // "Sensor (I2C/GPIO) is physically unresponsive"
    MemoryExhausted      = 0x63,       // "Node ran out of RAM trying to process this"
    StorageFull          = 0x64,       // Disk at capacity
    CriticalTemp         = 0x65        // Hardware is at a critical temperature


    // --- 0x80: Mesh/Transport Errors (Manager-Side Detected) ---
    PathLost             = 0x80,       // No route to destination
    LinkTimeout          = 0x81,       // Encrypted tunnel collapsed
    LinkCongested        = 0x82,       // "Medium is saturated, try later"
}
```
