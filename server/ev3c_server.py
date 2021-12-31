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


connections = []  # tuple: (ev3_port, controller_port)


def get_connected_port(port):
    global connections
    for connection in connections:
        if port == connection[0]:
            return connection[1]
        if port == connection[1]:
            return connection[0]
    raise UnconnectedPortException(port)


def get_port(server):
    return server.sockets[0].getsockname()[1] \
        if server.sockets is not [] else None


def get_ws_port(websocket):
    return get_port(websocket.ws_server)


def get_ports(client_servers):
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


async def get_client_port(range, client_servers, callback_on_message):
    used_ports = get_ports(client_servers)
    free_ports = [port for port in range if port not in used_ports]
    if free_ports == []:
        raise NoPortFreeException()
    client_port = free_ports[0]
    await start_server(client_servers, client_port, callback_on_message)
    return client_port


async def get_controller_port():
    global servers
    return await get_client_port(
        range(CONFIG['controller']['port_range']['start'],
              CONFIG['controller']['port_range']['end']),
        servers['controller'],
        on_controller_message
    )


async def get_ev3_port():
    global servers
    return await get_client_port(
        range(CONFIG['ev3']['port_range']['start'],
              CONFIG['ev3']['port_range']['end']),
        servers['ev3'],
        on_ev3_message
    )


def get_client_socket(opposite_websocket, client_servers):
    try:
        connected_port = get_connected_port(get_ws_port(opposite_websocket))
        for server in client_servers:
            if get_port(server) == connected_port:
                return server.websocket
    except UnconnectedPortException:
        pass
    raise UnconnectedClientException(opposite_websocket)


def get_ev3_socket(opposite_websocket):
    global servers
    return get_client_socket(opposite_websocket, servers['ev3'])


def get_controller_socket(opposite_websocket):
    global servers
    return get_client_socket(opposite_websocket, servers['controller'])


async def on_controller_message(websocket):
    async for message in websocket:
        ev3_socket = get_ev3_socket(websocket)
        await ev3_socket.send(message)


async def on_ev3_message(websocket):
    async for message in websocket:
        controller_socket = get_controller_socket(websocket)
        await controller_socket.send(message)


async def on_client_entry(websocket, callback_get_port):
    # TODO: If EV3 connects -> look for unbound controllers to connect to
    # TODO: If controller connects -> look for unbound EV3 to connect to
    # TODO: Password verification
    # initial_data = json.loads(await websocket.recv())
    try:
        # if not initial_data.get('password', None) == os.getenv('CLIENT_PW'):
        #     raise PermissionDeniedException("Entry password incorrect")
        assigned_port = await callback_get_port()
        await websocket.send(str(assigned_port))
    except NoPortFreeException:
        await on_client_reject(websocket)
    except PermissionDeniedException:
        await on_client_reject(websocket, "Permission denied")


async def on_client_reject(websocket, reason="Capacity reached"):
    await websocket.send("Rejected: {}".format(reason))


async def on_controller_entry(websocket):
    await on_client_entry(websocket, get_controller_port)


async def on_ev3_entry(websocket):
    await on_client_entry(websocket, get_ev3_port)


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
