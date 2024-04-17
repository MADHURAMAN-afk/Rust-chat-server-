// Importing necessary modules for our program to work
use std::io::{ErrorKind, Read, Write}; // We're bringing in some tools to help with input/output operations.
use std::net::TcpListener; // We're bringing in tools to create a listener for network connections.
use std::sync::mpsc; // We're bringing in tools to help with communication between different parts of our program.
use std::thread; // We're bringing in tools to work with threads, which allow our program to do multiple things at once.

// Defining constants for our program
const LOCAL: &str = "127.0.0.1:8090"; // We're setting up the address where our listener will be located.
const MSG_SIZE: usize = 32; // We're defining the maximum size of messages we can send and receive.

// Function to make the program pause for a short while
fn sleep() { // This is a function called "sleep".
    thread::sleep(::std::time::Duration::from_millis(100)); // It makes the program wait for a short time before continuing. small change
}

// The main part of our program
fn main() { // This is where our program starts.

    // Setting up our listener
    let server = TcpListener::bind(LOCAL).expect("listener failed to bind"); // We're creating a new listener at the address we specified.
    server.set_nonblocking(true).expect("failed to set non-blocking"); // We're configuring the listener to work without blocking the program's execution.

    // Setting up some containers to hold information
    let mut clients = vec![]; // We're creating a list to keep track of the computers that connect to us.
    let (tx, rx) = mpsc::channel::<String>(); // We're creating a channel for sending and receiving messages between different parts of our program.

    // A loop that keeps our program running indefinitely
    loop { // This loop keeps the program running forever.

        // Handling incoming connections
        if let Ok((mut socket, addr)) = server.accept() { // If a computer tries to connect to us...
            println!("client {} connected ", addr); // We print a message saying that the computer has connected.

            let tx = tx.clone(); // We make a copy of the sending part of our message channel.
            clients.push(socket.try_clone().expect("failed to clone client")); // We add the connected computer to our list of clients.

            // Spawning a new thread to handle communication with this client
            thread::spawn(move || loop { // We start a new thread to handle communication with this client.

                // Reading messages from the client
                let mut buff = vec![0; MSG_SIZE]; // We prepare a space to store messages from the client.

                match socket.read_exact(&mut buff) { // We try to read a message from the client.
                    Ok(_) => { // If we successfully read a message...
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>(); // We convert the message into text.
                        let msg = String::from_utf8(msg).expect("invalid utf message"); // We convert the text into a readable format.

                        println!("{}: {:?}", addr, msg); // We print out the message from the client.

                        tx.send(msg).expect("failed to send message to rx"); // We send the message to other parts of our program.
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (), // If there's no message from the client, we just continue.
                    Err(_) => { // If something goes wrong with reading the message...
                        println!("closing connection with : {}", addr); // We print out a message saying we're closing the connection.
                        break; // We stop talking to this client and exit the loop.
                    }
                }

                sleep(); // We take a short break before checking for more messages from the client.
            });
        }

        // Handling outgoing messages
        if let Ok(msg) = rx.try_recv() { // If there's a message waiting to be sent...
            clients = clients.into_iter().filter_map(|mut client| { // We go through our list of clients.
                let mut buff = msg.clone().into_bytes(); // We convert the message into bytes.
                buff.resize(MSG_SIZE, 0); // We make sure the message is the right size.

                client.write_all(&buff).ok(); // We send the message to this client.
                None // We remove this client from our list, as we've already sent them the message.
            }).collect(); // We collect the remaining clients back into our list.
        } sleep();
    }
}
