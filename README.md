Por el momento falta:
- DB
- Encripcion por TLS
- Logout
- Creacion de usuarios
- Manejo de los roles Driver Montitor y Admin
- Log de estatus
- Asignacion de Caminos
- Manejo de estados de Caminos
- Interfaz grafica
- Manejo de reconecciones
- Manejo de sessiones por medio de JWT

Por el momento el server corre en ==127.0.0.1:8089== en las direcciones /app y /ws 
/app sirve el cliente por medio de ==tower-http== , y wasm, el server hacepta connecciones 
y logins, maneja desconecciones y asignaciones de UserSessions las cuales contienen el 
rol,nombre,user id, y session token.

Estructura: 
==transp-common== contiene los elementos comunes entre el cliente y el servidor:
funciones de codificacion de mensajes web socket,
deficiones de Structs y Enums comunes como:
`LoginRequest`, `UserSession`, `User`, `Message` =="instrucciones de la api"==, etc.

el formato de los mensajes websocket es siempre un `WSMessage::Binary()`,
los mensajes permitidos son los definidos en ==transp-common== en el `enum Message`
==el formato de codificacion es postcard== usando serde.

los mensajes :
`postcard_codificador(Message::TipoMensaje(postcard_codificador(Estructura de datos)))`

para correr el servidor:
primero instalar cargo por medio de rustup https://rustup.rs/

en la terminal o windows cmd
ir a transp-server 
y compiar con : `cargo build --release`
y correr con: `cargo run --release`

PARA PODER CORRER EL SERVIDOR:
Se tiene que correr desde la carpeta transp-server si no los archivos del cliente no seran encontrados

si hay algun cambio en el cliente ir a transp-client
primero tener instalado wasm-pack y el target wasm32-unknown-unknown
`cargo install wasm-pack`
`rustup target add wasm32-unknown-unknown`

y compilar con `cargo build --release --target wasm32-unknown-unknown`
y producir ensamble final con `wasm-pack build --release --target web`
eliminar la carpeta transp-server/client/pkg y copiar la nueva carpeta
pkg en transp-client/pkg a transp-server/client
por ultimo correr el servidor
e ir a http://127.0.0.1:8089/app para ver los cambios





