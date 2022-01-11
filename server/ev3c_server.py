import asyncio
import json
import websockets
from dotenv import load_dotenv
from json.decoder import JSONDecodeError

CONFIG = {
    "ev3": {
        "port": 8800,
    },
    "controller": {
        "port": 8900,
    },
    "connection_url": "/ev3c",
    "env_path": ".env",
}

load_dotenv(CONFIG['env_path'])


class NoEV3AvailableError(Exception):
    """Raised when no EV3 is currently available to connect to."""
    def __init__(self, *args: object):
        super().__init__(
            "No EV3 is currently available", *args)


class PermissionDeniedError(Exception):
    """Raised when the client does not have permission for this \
        content/action."""
    def __init__(self, reason, *args: object):
        super().__init__(
            "Permission denied: {}".format(reason), *args)


class InitialDataError(Exception):
    """Raised when the clients initial does not have the expected format or content."""
    def __init__(self, message, inital_data_json=None, *args: object):
        self.message = "Faulty Initial Data: {}".format(message)
        self.json = inital_data_json
        super().__init__(self.message, *args)


connections = {
    # {ev3_id}: {
    #     "websocket": {ev3_websocket},
    #     "control": {controller_websocket},
    #     "queue": [{controller_websocket}]
    # }
}


def get_controled_websocket(controller_websocket):
    """Returns EV3 websocket the given controller websocket is controlling."""
    global connections
    for connection in connections.values():
        if connection['control'] == controller_websocket:
            return connection['websocket']
    return None


def get_controlling_websocket(ev3_websocket):
    """Returns controller websocket controlling the given EV3 controller."""
    global connections
    for connection in connections.values():
        if connection['websocket'] == ev3_websocket:
            return connection['control']

def get_connected_websockets(ev3_id):
    """Returns websockets of controlling and queued controller clients."""
    global connections
    controllers = []
    connection = connections[ev3_id]
    if connection['control'] is not None:
        controllers.append(connection['control'])
    controllers += connection['queue']
    return controllers

async def on_controller_connect(websocket):
    """Connects a controller to an EV3 and handles the data transfer."""
    try:
        await establish_controller_connection(websocket)
        # except PermissionDeniedError:
        #     await reject_client(websocket, "Permission denied")
    except JSONDecodeError:
        await reject_client(websocket, "No initial JSON data transfered")
    except NoEV3AvailableError:
        await reject_client(websocket, "No EV3 available")
    else:
        try:
            async for message in websocket:
                ev3_websocket = get_controled_websocket(websocket)
                if ev3_websocket is not None:
                    await ev3_websocket.send(message)
        finally:
            await disband_controller_connection(websocket)


async def establish_controller_connection(websocket):
    """Performs controller connection protocol. Blocks until connection is established.
    
    Raises
    ------
    NoEV3AvailableError:
        If there is no EV3 connected to the server.

    PermissionDeniedError:
        If the password is either not given or wrong.
    """
    global connections
    initial_data = json.loads(await websocket.recv())
    # TODO: Password verification
    # if not initial_data.get('password', None) == os.getenv('CLIENT_PW'):
    #     raise PermissionDeniedError("Entry password incorrect")    
    if len(connections.keys()) == 0:
        raise NoEV3AvailableError()
    preferred_ev3 = initial_data.get("preferred_ev3", None)
    if preferred_ev3 is None:
        preferred_ev3 = get_available_ev3()
    await control(preferred_ev3, websocket)


async def disband_controller_connection(websocket):
    """Removes websocket from its queue or removes websocket from control and yields control to first controller in queue."""
    global connections
    for ev3_id, connection in connections.items():
        if websocket == connection['control']:
            connection['control'] = None
            if len(connection['queue']) > 0:
                await control(ev3_id, connection['queue'][0])
            elif connection['websocket'] is None:
                del connections[ev3_id]
            break
        elif websocket in connection['queue']:
            connection['queue'].remove(websocket)
            break


async def control(ev3_id, websocket):
    """Sets websocket as controller of given EV3. Removes controller from queue, if present. \
        Puts controller in queue if the EV3 is already controlled."""
    global connections
    if connections[ev3_id]['control'] is None:
        connections[ev3_id]['control'] = websocket
        if websocket in connections[ev3_id]['queue']:
            connections[ev3_id]['queue'].remove(websocket)
        await update_client(websocket, "Control", ev3_id)
    else:
        connections[ev3_id]['queue'].append(websocket)
        await update_client(websocket, "Queue", ev3_id)


def get_available_ev3():
    """Returns ID of an EV3 that is either uncontrolled or has a the shortest queue."""
    global connections
    shortest_queue = (None, 0)
    for ev3_id, connection in connections.items():
        if connection['control'] is None:
            return ev3_id
        elif shortest_queue[0] is None or shortest_queue[1] < len(connection['queue']):
            shortest_queue = (ev3_id, len(connection['queue']))
    return shortest_queue[0]


async def on_ev3_connect(websocket):
    """Makes EV3 available to controllers and handles the data transfer."""
    try:
        await establish_ev3_connection(websocket)
    except JSONDecodeError:
        await reject_client(websocket, "No initial JSON data transfered")
    except InitialDataError as e:
        await reject_client(websocket, e.message)
    # except PermissionDeniedError:
    #     await reject_client(websocket, "Permission denied")
    else:
        try:
            async for message in websocket:
                controller_websocket = get_controlling_websocket(websocket)
                if controller_websocket is not None:
                    await controller_websocket.send(message)
        finally:
            await disband_ev3_connection(websocket)


async def establish_ev3_connection(websocket):
    """Performs EV3 connection protocol. Blocks until connection is established.
    
    Raises
    ------
    JSONDecoderError:
        If EV3 does not send JSON in its first message (as initial data).

    InitialDataError:
        If ID is missing in initial data sent by the EV3.
    
    PermissionDeniedError:
        If the password is either not given or wrong.
    """
    global connections
    initial_data = json.loads(await websocket.recv())
    # TODO: Password verification
    # if not initial_data.get('password', None) == os.getenv('CLIENT_PW'):
    #     raise PermissionDeniedError("Entry password incorrect")
    if "id" not in initial_data:
        raise InitialDataError("ID missing", initial_data)
    ev3_id = initial_data.get("id", None)
    ev3_connection = connections.get(ev3_id, {
        "websocket": None,
        "control": None,
        "queue": []
    })
    ev3_connection["websocket"] = websocket
    connections[ev3_id] = ev3_connection
    for controller in get_connected_websockets(ev3_id):
        if controller == ev3_connection['control']:
            await update_client(controller, "Control", ev3_id)
        else:
            await update_client(controller, "Queue", ev3_id)


async def disband_ev3_connection(websocket):
    """Disbands the EV3 connection and informs all connected controllers."""
    global connections
    for ev3_id, connection in connections.items():
        if websocket == connection['websocket']:
            controllers = get_connected_websockets(ev3_id)
            if controllers == []:
                del connections[ev3_id]
            else:
                for controller in controllers:
                    await update_client(controller, "Pending", ev3_id)
                connection['websocket'] = None
            break


async def reject_client(websocket, reason="Capacity reached"):
    await update_client(websocket, "Rejected", reason)


async def update_client(websocket, status, content):
    await websocket.send("{}: {}".format(status, content))


async def main():
    async with websockets.serve(on_controller_connect, "",
                                CONFIG["controller"]["port"]), \
            websockets.serve(on_ev3_connect, "", CONFIG["ev3"]["port"]):
        print("Connection entries on:")
        print("For EV3: {}:{}".format(CONFIG["ev3"]["port"],
                                      CONFIG["connection_url"]))
        print("For Controller: {}:{}".format(CONFIG["controller"]["port"],
                                             CONFIG["connection_url"]))
        await asyncio.Future()  # run forever


asyncio.run(main())
