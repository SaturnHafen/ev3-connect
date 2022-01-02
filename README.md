# ev3connect

## Server

### Establishing a Connection

#### For EV3 Clients

1. Connect to the `ev3 clients port`.
2. As first message, send a JSON string of the following format:

```json
{
    "id": "EV3 name"
}
```

- **id**: mandatory. Must be unique. If it's not, it will take over the established connections of another EV3 client with the same name.

3. Connection established. All following messages received come from a controller. The EV3 may now send messages to the connected controller.

#### For Controller Clients

1. Connect to the `controller clients port`.
2. As first message, send a JSON string of the following format:

```json
{
    "preferred_ev3": "EV3 name"
}
```

- **preferred_ev3**: Optional. If present, the controller will be connected to this EV3. If **no** EV3 with this ID is available, `preferred_ev3` is treated as if it were not present. If `preferred_ev3` is **not** present, it is connected to the best available EV3 (best available: either an EV3 that had no controller yet or has the shortest queue of waiting controllers).

3. Connection established. Controller receives a message of the format `Status: EV3_ID`.

- `Status`: Either `Control` or `Queue`. `Control` means the controller can now communicate bidirectionally with the EV3 client. This can be done until either side closes the connection. `Queue` means the controller is designated to control an EV3, but this EV3 is still in control of another controller. Messages sent to the EV3 are lost. As soon as the EV3 is not controlled any more and all other queued controllers before have disconnected, the controller receives a status update like `Control: EV3_ID`.
- `EV3_ID`: ID of the EV3 the controller is connected to.

### Disbanding a Connection

If an EV3 disbands its connection, all connected clients (either in control or in queue) receive the status update `Pending: EV3_ID`. Messages sent to the EV3 are now lost. All clients can keep or disband their connection. Clients who keep their connection receive a status update (`Control`/`Queue`) as soon as the EV3 client established its connection again.

If a controller disbands its connection, it is either removed from the queue or, if currently in control, yields its control to the first controller in the EV3s queue.