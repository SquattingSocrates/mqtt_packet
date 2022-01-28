# Mqtt message decoding/encoding library

This is a library designed to be used for creating mqtt clients or mqtt brokers.
As many things as were reasonable are encoded in the type system, e.g.
- packets have their own types, except for PUBCOMP/PUBREC/PUBREL/PUBACK, since they are essentially the same
- reason codes are enums and it's not possible to build a packet with an invalid reason code
- properties are defined for every single packet type and therefore only valid property codes can be written into the packet

### Supports: MQTTv3 and MQTTv5

Messages of these versions should be decodable/encodable with this library.

### What works so far (Implemented and tested):

| Encode | Decode | Packet Type |
|--------|--------|-------------|
| ✅     | ✅      | Connect     |
| ✅     | ✅      | Connack     |
| ✅     | ✅      | Publish     |
| ✅     | ✅      | Puback      |
| ✅     | ✅      | Pubrec      |
| ✅     | ✅      | Pubrel      |
| ✅     | ✅      | Pubcomp     |
| ✅     | ✅      | Subscribe   |
| ✅     | ✅      | Suback      |
| ✅     | ✅      | Unsubscribe |
| ✅     | ✅      | Unsuback    |
| ✅     | ✅      | Pingreq     |
| ✅     | ✅      | Pingresp    |
| ✅     | ✅      | Disconnect  |
| ✅     | ✅      | Auth        |

-------------------------

##### However certain things still need to be added/improved:


- [ ] A better command building API?
- [ ] Support for Maximum Packet Size (MQTTv5). Should not send certain properties if they "bloat" the packet
- [ ] ...