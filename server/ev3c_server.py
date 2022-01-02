import asyncio
import json
import os
import websockets
from dotenv import load_dotenv

CONFIG = {
    "ev3": {
        "entry_port": 8900,
        "port_range": {
            "start": 6790,
            "end": 6820
        }
    },
    "controller": {
        "entry_port": 8800,
        "port_range": {
            "start": 6890,
            "end": 6920
        }
    },
    "connection_url": "/ev3c",
    "web_backend_command": {
        "set": "uberspace web backend set {} --http --port {}",
        "del": "uberspace web backend del {} --port {}"
    },
    "env_path": ".env",
    "uberspace_active": False
}

load_dotenv(CONFIG['env_path'])


class NoPortFreeException(Exception):
    """Raised when no port is currently available to be assigned to \
        the client."""
    def __init__(self, message="Currently no free port available.",
                 *args: object):
        super().__init__(message, *args)


class UnconnectedClientException(Exception):
    """Raised when no EV3/Controller is currently connected the websocket."""
    def __init__(self, websocket, *args: object):
        super().__init__(
            "No client is currently connected to port {}".format(get_ws_port(websocket)), *args)


class NoFreeEV3Exception(Exception):
    """Raised when no EV3 is currently available to connect to."""
    def __init__(self, *args: object):
        super().__init__(
            "No EV3 is currently available", *args)


class UnconnectedPortException(Exception):
    """Raised when port is not connected to another port."""
    def __init__(self, port, *args: object):
        super().__init__(
            "No port currently connected to port {}".format(port), *args)


class PermissionDeniedException(Exception):
    """Raised when the client does not have permission for this \
        content/action."""
    def __init__(self, reason, *args: object):
        super().__init__(
            "Permission denied: {}".format(reason), *args)


servers = {
    'controller': [],
    'ev3': []
}


connections = []  # tuple: (ev3_websocket, controller_websocket)


def get_port(server):
    return server.sockets[0].getsockname()[1] \
        if server.sockets is not [] else None


def get_ws_port(websocket):
    return get_port(websocket.ws_server)


def get_ports(client_servers):
    """Returns ports the given client servers listen on."""
    ports = []
    for server in client_servers:
        ports.append(get_port(server))
    return ports


async def start_server(server_group, port, callback_on_message):
    if CONFIG['uberspace_active']:
        os.system(CONFIG['web_backend_command']['set'].format(
            CONFIG['connection_url'], port))
    server_group.append(await websockets.serve(callback_on_message, "", port))


async def stop_server(server):
    await server.close()
    await server.wait_closed()
    if CONFIG['uberspace_active']:
        os.system(CONFIG['web_backend_command']['del'].format(
            CONFIG['connection_url'], get_port(server)))
    return server


def get_ev3_socket(controller_socket):
    """Returns the EV3 socket currently connected to given controller socket."""
    global connections
    for connection in connections:
        if controller_socket == connection[1]:
            return connection[0]
    raise UnconnectedClientException(controller_socket)


def get_controller_socket(ev3_socket):
    """Returns the controller socket currently connected to given EV3 socket."""
    global connections
    for connection in connections:
        if ev3_socket == connection[0]:
            return connection[1]
    raise UnconnectedClientException(ev3_socket)


async def on_controller_connect(websocket):
    """Connects a controller to a free EV3 and handles the data transfer."""
    try:
        open_controller_connection(websocket)
        async for message in websocket:
            controller_socket = get_ev3_socket(websocket)
            await controller_socket.send(message)
    finally:
        on_client_reject(websocket, "No free EV3 available")
        close_connection(websocket)


async def on_ev3_connect(websocket):
    """Makes EV3 available to controllers and handles the data transfer."""
    try:
        open_ev3_connection(websocket)
        async for message in websocket:
            controller_socket = get_controller_socket(websocket)
            await controller_socket.send(message)
    finally:
        close_connection(websocket)


def open_controller_connection(websocket):
    global connections
    for index, connection in enumerate(connections):
        if connection[1] is None:
            connections[index] = (connection[0], websocket)
            return
    raise NoFreeEV3Exception()


def open_ev3_connection(websocket):
    global connections
    connections.append((websocket, None))


def close_connection(websocket):
    """Removes the websocket from connections."""
    global connections
    for index, connection in enumerate(connections):
        if websocket == connection[0]:
            connections[index] = (None, connection[1])
            return
        if websocket == connection[1]:
            connections[index] = (connection[0], None)
            return


async def get_new_client_port(range, client_servers, callback_on_message):
    """Opens and returns a port in the given port range. Also starts a server on this port."""
    used_ports = get_ports(client_servers)
    free_ports = [port for port in range if port not in used_ports]
    if free_ports == []:
        raise NoPortFreeException()
    client_port = free_ports[0]
    await start_server(client_servers, client_port, callback_on_message)
    return client_port


async def get_new_controller_port():
    """Opens and returns a port from the Controller port range. Also starts a server on this port."""
    global servers
    return await get_new_client_port(
        range(CONFIG['controller']['port_range']['start'],
              CONFIG['controller']['port_range']['end']),
        servers['controller'],
        on_controller_connect
    )


async def get_new_ev3_port():
    """Opens and returns a port from the EV3 port range. Also starts a server on this port."""
    global servers    
    ev3_port = await get_new_client_port(
        range(CONFIG['ev3']['port_range']['start'],
              CONFIG['ev3']['port_range']['end']),
        servers['ev3'],
        on_ev3_connect
    )
    return ev3_port


async def on_client_entry(websocket, callback_get_new_port):
    """Start server for a client on a port retrieved from respective range. \
        The client can later connect to this server to send and receive messages."""
    # TODO: Password verification
    # initial_data = json.loads(await websocket.recv())
    try:
        # if not initial_data.get('password', None) == os.getenv('CLIENT_PW'):
        #     raise PermissionDeniedException("Entry password incorrect")
        assigned_port = await callback_get_new_port()
        await websocket.send(str(assigned_port))
    except NoPortFreeException:
        await on_client_reject(websocket)
    # except PermissionDeniedException:
    #     await on_client_reject(websocket, "Permission denied")


async def on_client_reject(websocket, reason="Capacity reached"):
    await websocket.send("Rejected: {}".format(reason))


async def on_controller_entry(websocket):
    """Start server on a port in the controller port range which the controller client can connect to later."""
    await on_client_entry(websocket, get_new_controller_port)


async def on_ev3_entry(websocket):
    """Start server on a port in the EV3 port range which the EV3 client can connect to later."""
    await on_client_entry(websocket, get_new_ev3_port)


async def main():
    async with websockets.serve(on_controller_entry, "",
                                CONFIG["controller"]["entry_port"]), \
            websockets.serve(on_ev3_entry, "", CONFIG["ev3"]["entry_port"]):
        print("Connection entries on:")
        print("For EV3: {}:{}".format(CONFIG["connection_url"],
                                      CONFIG["ev3"]["entry_port"]))
        print("For Controller: {}:{}".format(CONFIG["connection_url"],
                                             CONFIG["controller"]["entry_port"]))
        global servers
        await asyncio.Future()  # run forever
        # TODO: Handle disconnect
        for server in servers['ev3'] + servers['controller']:
            stop_server(server)


asyncio.run(main())
