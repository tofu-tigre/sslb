use std::{io, error::Error, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use crate::policy::Policy;

const MAX_REQUEST_SIZE: usize = 1024 * 1000;

pub struct LoadBalancer {
  endpoints: Vec<String>,
  listener: TcpListener,
  policy: Box<dyn Policy<String>>,
}

unsafe impl Send for LoadBalancer {}

impl LoadBalancer {
  pub async fn build(
    ip: &str,
    endpoints: Vec<String>,
    policy: Box<dyn Policy<String> + Send>)
    -> Result<LoadBalancer, io::Error> {
    assert!(endpoints.len() > 0);
    let listener = TcpListener::bind(ip).await?;
    Ok(LoadBalancer { endpoints, listener, policy })
  }

  pub async fn run(&mut self) -> () {
    loop {
      // Wait for incoming connections.
      let connection = self.listener.accept().await;
      if connection.is_err() {
        println!("ERROR: {}", connection.unwrap_err());
        continue;
      }

      // Choose which endpoint to forward to.
      let endpoint = self.policy.select(&self.endpoints);
      println!("Selected endpoint {}", endpoint);

      // Hand off forwarding to seperate task.
      tokio::spawn(async move {
        let (stream, socket) = connection.unwrap();
        println!("Accepted user connection {}.", socket);

        match handle_connection(endpoint, stream, socket).await {
          Ok(_) => (),
          Err(err) => eprintln!("ERROR: {}", err),
        }
      });
    }
  }
}

async fn handle_connection(
  endpoint: String,
  mut user_connection: TcpStream,
  _user_addr: SocketAddr)
  -> Result<(), Box<dyn Error>> {
  // Establish connection with endpoint.
  let mut endpoint_connection = TcpStream::connect(&endpoint).await?;
  println!("Endpoint connection {} accepted.", endpoint);
  // Read in user connection data.
  let usr_buf = read_into_buffer(&mut user_connection).await?;

  // Write user request to endpoint.
  endpoint_connection.write_all(&usr_buf).await?;

  // Wait for endpoint response.
  let serv_buf = read_into_buffer(&mut endpoint_connection).await?;

  // Write endpoint response back to user.
  user_connection.write_all(&serv_buf).await?;
  Ok(())
}

async fn read_into_buffer(src: &mut TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
  let mut buf = vec![0u8; 1024];
  loop {
    // println!("READING");
    // println!("{:#?}", buf);
    let bytes_read = src.read(&mut buf).await?;
    if bytes_read == 0  {
      return Err("EOF found while reading request")?
    }
    if buf[bytes_read - 2] == '\r' as u8 && buf[bytes_read - 1] == '\n' as u8 {
      break;
    }
    if buf.len() == buf.capacity() {
      if buf.capacity() * 2 >= MAX_REQUEST_SIZE {
        return Err(format!(
          "Request exceeded max allowable size ({}B)",
          MAX_REQUEST_SIZE))?
      }
      buf.reserve(buf.capacity() * 2);
    }
  }
  Ok(buf)
}