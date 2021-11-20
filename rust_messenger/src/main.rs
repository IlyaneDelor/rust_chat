use std::process;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};
#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // Création du listener
    // Quand on utilise .await, on dit qu'au compilateur qu'il peut attendre que
    // la partie à gauche (donc ici TcpListenenr.bind) ait fini et retourne quelque chose
    // Ensuite comme l'async est wrappé, il faut le déwrapper
    let listener = match TcpListener::bind("localhost:8080").await {
        Ok(listener) => listener,
        Err(_e) => process::exit(1),
    };

    let (tx, _rx) = broadcast::channel(10);

    // Cette loop permet de handle plusieurs clients simulatanément
    loop {
        let (mut socket, addr) = match listener.accept().await {
            Ok((socket, addr)) => (socket, addr),
            Err(_e) => process::exit(1),
        };

        println!("Connection establish");

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // On move chaque client handling dans son task indépendant
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            // loop qui permet de lire des messages à la suite (et ne pas s'arrêter quand un message est envoyé)
            loop {
                // select permet de run plusieurs async en même temps
                // de manière concurrentes
                tokio::select! {

                    // reader va run, envoyer dans result le résultat puis run le code après =>
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }

                        //tx.send((line.clone(), addr)).unwrap();
                        let cl = match tx.send((line.clone(),addr)) {
                            Ok(cl) => cl,
                            Err(_e) => process::exit(1),
                        };
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (mut msg, other_addr) = match result {
                            Ok((msg,other_addr)) =>(msg,other_addr),
                            Err(_e) => process::exit(1),
                        };

                        // Pour éviter d'avoir un écho de notre propre message
                        // => testé en telnet
                        if addr != other_addr {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
