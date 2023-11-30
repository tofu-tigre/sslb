use log::info;
use sslb::lb::LoadBalancer;
use sslb::policy::{SimpleRoundRobinPolicy, Policy};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const HTTP_OK_RESPONSE: &'static str = "HTTP/1.1 200 OK\r\n";
const HTTP_GET_REQUEST: &'static str = "GET / HTTP/1.1\r\n";
const HTTP_SERVICE_UNAVAILABLE: &'static str =
  "HTTP/1.1 503 Service Unavailable Content-Type: text/plain

  503 Service Unavailable
  The server is temporarily unable to service your request due to maintenance downtime or capacity problems. Please try again later.\r\n";


async fn create_dummy_endpoint(addr: &str) {
  info!("Create dummy endpoint with addr: {}", addr);
  let listener = TcpListener::bind(addr).await.unwrap();
  tokio::spawn(async move {
    loop {
      let (mut stream, _) = listener.accept().await.unwrap();
      let mut buf = vec![0u8; 1024];
      stream.read(&mut buf).await.unwrap();
      let _ = stream.write_all(HTTP_OK_RESPONSE.as_bytes()).await.unwrap();
    }
  });
}

async fn create_lb_with_policy(addr: &str, policy: Box<dyn Policy>) {
  let mut server = LoadBalancer::build(addr, policy).await.unwrap();
  tokio::spawn(async move {
    server.run().await.unwrap();
  });
}

#[tokio::test]
async fn load_balancer_works() {
  let endpoints = vec![
    "localhost:8001".to_string(),
    "localhost:8002".to_string(),
    "localhost:8003".to_string(),
    "localhost:8004".to_string()];
  for endpoint in &endpoints {
    create_dummy_endpoint(endpoint).await;
  }

  let server_addr = "localhost:8000";
  let policy =
    Box::new(SimpleRoundRobinPolicy::new(endpoints.into_iter().collect()));
  create_lb_with_policy(server_addr, policy).await;

  let mut stream = TcpStream::connect(server_addr).await.unwrap();
  stream.write_all(HTTP_GET_REQUEST.as_bytes()).await.unwrap();
  let mut buf = vec![0u8; 1024];
  let _bytes_read = stream.read(&mut buf).await.unwrap();
  assert_eq!(HTTP_OK_RESPONSE, std::str::from_utf8(&buf).unwrap().trim_end_matches("\0"));
}

#[tokio::test]
async fn load_balancer_fails_when_all_endpoints_disconnect() {
  let endpoints = vec![
    "localhost:8001".to_string(),
    "localhost:8002".to_string(),
    "localhost:8003".to_string(),
    "localhost:8004".to_string()];
  let server_addr = "localhost:8000";
  let policy =
    Box::new(SimpleRoundRobinPolicy::new(endpoints.into_iter().collect()));

  let mut server = LoadBalancer::build(server_addr, policy).await.unwrap();
  tokio::spawn(async move {
    server.run().await.unwrap_err();
  });

  let mut stream = TcpStream::connect(server_addr).await.unwrap();
  stream.write_all(HTTP_GET_REQUEST.as_bytes()).await.unwrap();
  let mut buf = vec![0u8; 1024];
  let _bytes_read = stream.read(&mut buf).await.unwrap();
  assert_eq!(HTTP_SERVICE_UNAVAILABLE, std::str::from_utf8(&buf).unwrap().trim_end_matches("\0"));
}