use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

//Taille d'un message
const MSG_SIZE: usize = 32;

fn main() {
    //Essaie de connection en localhost
    let server = TcpListener::bind("localhost:8080").expect("Le server n'arrive pas se connecter");

    //les opérations de lecture, d'écriture, de réception et d'envoi deviendront non bloquantes,
    //c'est-à-dire qu'elles reviendront immédiatement de leurs appels.

    server
        .set_nonblocking(true)
        .expect("N'arrive pas a passé en mode non-blocking");

    //OK ou io::ErrorKind::WouldBlock

    //Sender<String>
    let (sending, receive) = mpsc::channel::<String>();

    //Tableau de Sockets
    let mut tab_socket = vec![];

    //Boucle qui continue jusqu'au break d'arret du serveur
    loop {
        /*Accepte une nouvelle Connexion d'un client au serveur
         */
        if let Ok((mut socket, addr)) = server.accept() {
            /*Message d'accueil
             */
            println!("Connexion Réussi, Bienvenue {}", addr);
            /*
                Clone "Sending" pour pouvoir immplementer le trait Copie qui n'est pas sur le type Sender
            */

            let sending = sending.clone();

            //On ajoute dans le tableau des sockets celui dui client qui vient de ce connecter

            tab_socket.push(socket.try_clone().expect("failed to clone client"));

            //Thread = Pour que des parties independantes fonctionne simultanement
            //move = La data n'est plus valide

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                //
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                        println!("-------------------------");

                        println!("{}: {:?}", addr, msg);
                        sending.send(msg).expect("f");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("La connecton a été interrompu avec : {}", addr);
                        break;
                    }
                }

                thread::sleep(::std::time::Duration::from_millis(150));
            });
        }

        if let Ok(msg) = receive.try_recv() {
            tab_socket = tab_socket
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }

        thread::sleep(::std::time::Duration::from_millis(150));
    }
}
