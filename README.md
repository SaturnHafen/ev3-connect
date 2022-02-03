# ev3-connect

Program and control physical LEGO MINDSTORMS EV3s remotely over the Internet.

## What is ev3-connect?

This collection of tools allows one or multiple users to connect the usual [LEGO MINDSTORMS LabView programming environment](https://education.lego.com/de-de/downloads/retiredproducts/mindstorms-ev3-lab/software) to one or multiple EV3s via the Internet.  
In other words: EV3s in another part of the world can be programmed and controlled as if they were right next to you. And not only by you, but by different users, thus also suitable for situations where there are more users than EV3s.

## Why would you want ev3-connect?

As of 2022 the world is still pretty much affected by the Covid pandemic, so presence lectures for students are not the best option. The only alternative for EV3 programming courses: simulators. But just staring at a simulator next to your programming environment is quite boring and does not offer the same learning experience as a physical robot.

The ev3-connect software has a better solution:

The EV3s are stationed at the lecturer's home (with a camera pointed to them). With ev3-connect, the students can use the [LEGO MINDSTORMS LabView software](https://education.lego.com/de-de/downloads/retiredproducts/mindstorms-ev3-lab/software) to program one of the EV3s as usual and see the video stream of their robot. Ev3-connect ensures a rotation of programming access, so multiple students can work with one EV3.

## How does it work?

The ev3-connect setup consists of three softwares:

- a **controller client**, run on the computer with the LEGO MINDSTORMS programming environment
- a **server**, which routes the commands from the controller client to
- an **EV3 client**, run on the computer that is connected to the EV3 via Bluetooth and passes the commands to the EV3 (and vice-versa)

The best: ev3-connect just passes the commands from the programming environment to the EV3 and vice-versa, thus supporting all features of a direct connection of environment and EV3.

See [Technical Details](#technical-details) for an in-depth explanation.

## Requirements

### Server Side

**ToDo**

### EV3 Side

1. A Linux (tested) or Windows (currently not tested) PC with Bluetooth capabilities
2. A stable internet connection
3. Recommended: A webcam to transmit the video stream (the transmission part is not part of this software)
4. At least one EV3

### Controller Side

1. A Windows Machine (Virtual Maschine should work)
2. A stable internet connection

## Getting Started

### Server

**ToDo**

### EV3 Client - ev3-tunnel-exit

1. Download the `ev3` (*Linux*) or `ev3.exe` (*Windows*) binary and the `ev3.config.toml.example` file from the latest [release](https://github.com/SaturnHafen/ev3-connect/releases)
2. *On Windows:* You may have to convince Windows defender / Microsoft Edge that the file is not malicious :)
3. Rename `ev3.config.toml.example` to `config.toml` (you might need to enable filename extensions in the Windows Explorer)
4. Open `config.toml` and edit the details to match your (server) setup
5. Start your EV3 and enable Bluetooth on it
7. *On first connection only*: Pair to the EV3 using your system's bluetooth management software (probably somewhere in the setting)
8. Start the `ev3`/`ev3.exe` binary (Windows Defender may warn you about the software being malicious - just execute anyways)
9. Your EV3 should now connect to the server and wait for a controller to connect

### Controller Client - ev3-tunnel-entry

1. Download the `controller.exe` binary and the `controller.config.toml.example` file from the latest [release](https://github.com/SaturnHafen/ev3-connect/releases)
2. *On Windows:* You may have to convince Windows defender / Microsoft Edge that the file is not malicious :)
3. Rename `controller.config.toml.example` to `config.toml` (you might need to enable filename extensions in the windows explorer)
4. Open `config.toml` and edit the details to match your (server) setup.
5. Start the `controller.exe` and LEGO MINDSTORMS LabView (a specific order should not be necessary. Windows Defender may warn you about the software being malicious - just execute anyways)
6. If an EV3 is connected to the server, it should be listed in LabView's connection panel
7. Connect to the EV3 as usual and start programming

## Technical Details

### Controller Client

1. The controller client creates an encrypted WebSocket connection (`wss://`) to the server (specified inside `cargo.toml`)
2. The controller client requests a specific EV3 (specified inside `cargo.toml`)
3. *If the requested EV3 is not available:* You receive control over an other EV3 or you are placed inside a queue for when the currently controlling person disconnects
4. *If the requested EV3 is available:* You receive control over the EV3 and LEGO LabView will show the available EV3 inside the bottom right connection panel
5. Connect to the EV3 inside LabView
7. Every feature of LEGO LabView should work as expected (e.g. `Upload & Run`, Overview over connected sensors, ...)

### EV3 Client

1. The EV3 client creates a connection to the specified EV3 (specified inside `config.toml`) via Bluetooth
2. The EV3 Client creates an encrypted WebSocket connection (`wss://`) to the server (specified inside `config.toml`)
3. Every command received from the websocket connection will be relayed to the EV3 and every response (if there is one) will be relayed back to the websocket connection

## Developing

### endpoints (controller & ev3 side)

1. Download the repository via command-line git or the green button in the top right
2. Make your changes
3. use cargo build to compile your changes (*debug build*)
4. use cargo run to build and run your changes (*debug build*; Caution: You need to put your `config.toml` inside the build output directory, otherwise you might not be able to connect to your expected services)

## Problems & How to fix them

### My Bluetooth adapter does not connect to the EV3

Try to connect first via your OS Bluetooth functionality (to finish the handshake). After that try again to connect. Also remember to put your bluetooth serial-number into the `config.toml` file.

**Linux users**: Sometimes your Bluetooth interface might be blocked by `rfkill`. Use `rfkill list` to check that. If your bluetooth is softblocked use `rfkill unblock bluetooth` to reenable the interface. (You need to be `sudo` to do that)

### My Software tries to connect to `localhost` / `strange port` / ...

Place your changed `config.toml` in the *same* directory as the executable and change your current working directory to that directory too. Also make sure that you saved your changes.

## Acknowledgments

Thanks to
 - https://ev3directcommands.blogspot.com/ for providing an insight into the inner workings of the EV3 protocol
 - http://www.monobrick.dk/guides/how-to-establish-a-wifi-connection-with-the-ev3-brick/ for providing the documentation on how to connect to the EV3 via WiFi (*page is transmitted via http*)
 - [SilvanVerhoeven](https://github.com/SilvanVerhoeven) for listening to me while i was solving stupid bugs and developing the server :)
 - [Lego Education](https://education.lego.com/de-de/product-resources/mindstorms-ev3/downloads/developer-kits) for providing the source code / reference material for the EV3
