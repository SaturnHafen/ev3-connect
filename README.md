# ev3-connect

## Why?

As of 2022 the world is still pretty much in Covid-Lockdown so presence-lectures for students are not a real option. Some lectures are still taking place like our ev3-robotics course (but obviously not in presence). But just staring at a simulator window next to your Programming Software is quite boring (and does not work for everybody), so here is a better solution:

The EV3s are stationed at the lecturers home (with a camera pointed to them) and the ev3-tunnel-exit software running. The students use the [legacy Lego-Labview software](https://education.lego.com/de-de/downloads/retiredproducts/mindstorms-ev3-lab/software) and the ev3-tunnel-entry software. With the software started a robot will show up inside Lego-LabView (visualized as if it is connected via Wifi) and can be programmed like a standard Lego Mindstorms EV3. The data is transmitted via an encrypted Websocket connection to the server and then to the connected EV3 (which then executes the program).

## Requirements

### ev3 side

 1. A Linux (tested) or Windows (currently not tested) PC with Bluetooth capabilities
 2. A stable internet connection
 3. A webcam to transmit the videostream (the transmission part is not part of this software)
 4. At least one EV3

### controller side

1. A Windows Machine (Virtual Maschine should work)
2. A stable internet connection

## How to use the software?

### ev3-tunnel-exit (physical robot side, "ev3")

1. Download the `ev3` (*Linux*) or `ev3.exe` (*Windows*) binary and the `ev3.config.toml.example` file from the latest [release](https://github.com/SaturnHafen/ev3-connect/releases)
2. *Windows only* Convince Windows defender / Microsoft Edge that the file is not malicious :)
3. rename `ev3.config.toml.example` to `config.toml` (you might need to enable filename extensions in the windows explorer)
4. open `config.toml` and edit the details that they match your setup.
5. Start your EV3 and enable bluetooth on them
6. *First connection only*: Pair to the EV3 using your system bluetooth management software (probably somewhere in the setting)
7. Start the Program

### ev3-tunnel-entry (student side "controller")

1. Download the `controller.exe` binary and the `controller.config.toml.example` file from the latest [release](https://github.com/SaturnHafen/ev3-connect/releases)
2. Convince Windows defender / Microsoft Edge that the file is not malicious :)
3. rename `controller.config.toml.example` to `config.toml` (you might need to enable filename extensions in the windows explorer)
4. open `config.toml` and edit the details that they match your setup.
5. Start the software and Lego-LabView (no specific order should be necessary)

## How does it work?

### controller side

1. The software creates an encrypted WebSocket (`wss://`) connection to the specified server (specified inside `cargo.toml`)
2. The software requests a specific EV3 (specified inside `cargo.toml`)
3. *if the requested EV3 is not available* You receive control over an other EV3 or you are placed inside a queue for when the currently controling person disconnects
4. *if the requested EV3 is available* You receive control over the EV3 and Lego-LabView will show an available EV3 inside the bottom right connection panel
5. Connect to the EV3
6. Every feature of Lego-LabView should work as expected (e.g. `Upload & Run`, Overview over connected sensors, ...)

### ev3 side

1. The software creates a connection to the specified EV3 (specified inside `config.toml`) via Bluetooth
2. The software creates an encrypted WebSocket (`wss://`) connection to the specified server (specified inside `config.toml`)
3. Every command received from the websocket connection will be relayed to the EV3 and every response (if there is one) will be relayed back to the websocket connection

## Developing

### endpoints (controller & ev3 side)

1. Download the repository via command-line git or the green button in the top right
2. Make your changes
3. use cargo build to compile your changes (*debug build*)
4. use cargo run to build and run your changes (*debug build*; Caution: You need to put your `config.toml` inside the build output directory, otherwise you might not be able to connect to your expected services)

## Problems & How to fix them

> My Bluetooth adapter does not connect with the EV3

Try to connect first via your OS Bluetooth functionality (to finish the handshake). After that try again to connect. Also remember to put your bluetooth serial-number into the `config.toml` file.

**Linux users**: Sometimes your Bluetooth interface might be blocked by `rfkill`. Use `rfkill list` to check that. If your bluetooth is softblocked use `rfkill unblock bluetooth` to reenable the interface. (You need to be `sudo` to do that)

---

> My Software tries to connect to `localhost` / `strange port` / ...

Place your changed `config.toml` in the *same* directory as the executable and change your current working directory to that directory too. Also make sure that you saved your changes.

## Acknowledgments

Thanks to
 - https://ev3directcommands.blogspot.com/ for providing an insight into the inner workings of the EV3-Protocol
 - http://www.monobrick.dk/guides/how-to-establish-a-wifi-connection-with-the-ev3-brick/ for providing the dokumentation on how to connect to the ev3 via wifi (*page is transmitted via http*)
 - [Silvan](https://github.com/SilvanVerhoeven) for listening to me while i was solving stupid bugs :)
 - [Lego Education](https://education.lego.com/de-de/product-resources/mindstorms-ev3/downloads/developer-kits) for providing the source code / reference material for the EV3
