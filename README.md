# ClientServer
A client/server implemented in Rust


This client-server application in Rust is my first Rust application.
It is a client/server using Rust's tcp/ip, where clients will connect to the server and register to receive messages.

The purpose of this application is for a server to send messages to several clients. In my lab I had the following problem: some Raspberry Pi servers are being powered by a UPS. When the power from the utility company goes out and the UPS starts working, the battery starts to discharge and at some point in the future the machines will go down due to lack of power when the battery drops below the threshold voltage to power the UPS.

That said, the server of this application will monitor the battery voltage and when it reaches a critical value to power the machines, the server will send a warning message to shut down each client registered on the server.

