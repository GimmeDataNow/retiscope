# Retiscope Protocol Specification (v0.1.0)

This document defines the wire format for Retiscope telemetry and management data over the Reticulum Network Stack (RNS).

## Announce Format

All Retiscope-enabled nodes must include a specific app_data payload in their RNS Announces to be discoverable by the Retiscope Visualizer.

**Format**: **RS**|**VERSION**|**FLAGS**|**OTHER_DATA**

| Component  | Type         |	Description                                          |
| ---------- | ------------ | ---------------------------------------------------- |
| RS         | String       | Static header identifying the Retiscope protocol.    |
| VERSION    | SemVer       |	The protocol version (e.g., 0.1.0).                  |
| FLAGS      | Bitmask (u8) |	Node capabilities and status.                        |
| OTHER_DATA | Optional     | Context-specific data (e.g., Battery, Load, or Name).|


**Flags**:
| Bit        | Name             |	Description                                          |
| ---------- | ---------------- | ---------------------------------------------------- |
| 0x01       |	IS_SERVER       |	Node serves data/logs (Retiscope Service).           |
| 0x02       |	TRUSTED_ONLY    |	Connection requires Identity authentication.         |
| 0x03       |	MFA_REQUIRED    |	MFA signing may be required to access all feature.   |
| 0x04       |	REQUIRES_AUTH   |	Connection requires Password authentication.         |
| 0x05       |	MANAGEABLE	    | Node accepts remote configuration commands.          |
| 0x08       |	LOW_B_W	        | Node is on a low-bandwidth link (LoRa/HF).           |
