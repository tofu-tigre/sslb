use std::{io, error::Error, net::SocketAddr};
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use crate::policy::Policy;

const MAX_REQUEST_SIZE: usize = 1024 * 1000;
const HTTP_SERVICE_UNAVAILABLE: &'static str =
  "HTTP/1.1 503 Service Unavailable Content-Type: text/plain

  503 Service Unavailable
  The server is temporarily unable to service your request due to maintenance downtime or capacity problems. Please try again later.\r\n";

pub struct LoadBalancer {
  listener: TcpListener,
  policy: Box<dyn Policy>,
}

unsafe impl Send for LoadBalancer {}

impl LoadBalancer {
  pub async fn build(
    ip: &str,
    policy: Box<dyn Policy + Send>)
    -> Result<LoadBalancer, io::Error> {
    let listener = TcpListener::bind(ip).await?;
    Ok(LoadBalancer { listener, policy })
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    loop {
      // Wait for incoming connections.
      let connection = self.listener.accept().await;
      if let Err(err) = connection {
        error!("{}", err);
        continue;
      }

      let (mut stream, socket) = connection.unwrap();
      info!("Accepted user connection {}.", socket);

      // Choose which endpoint to forward to.
      let mut endpoint;
      let endpoint_connection;
      loop {
        endpoint = match self.policy.select(&socket.to_string()) {
          Some(e) => e,
          None => {
            stream.write_all(HTTP_SERVICE_UNAVAILABLE.as_bytes()).await?;
            let _ = stream.shutdown().await;
            return Err("All endpoints dropped")?
          },
        };
        info!("Selected endpoint {}", endpoint);

        endpoint_connection = match TcpStream::connect(&endpoint).await {
          Ok(v) => v,
          Err(err) => {
            error!("{}", err);
            self.policy.remove(&endpoint);
            continue;
          },
        };
        info!("Endpoint connection {} accepted.", endpoint);
        break;
      }

      // Hand off forwarding to seperate task.
      tokio::spawn(async move {
        match handle_connection(endpoint_connection, stream, socket).await {
          Ok(_) => (),
          Err(err) => error!("{}", err),
        }
      });
    }
  }
}

async fn handle_connection(
  mut endpoint_connection: TcpStream,
  mut user_connection: TcpStream,
  _user_addr: SocketAddr)
  -> Result<(), Box<dyn Error>> {
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
      return Err("EOF encountered while reading request")?
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