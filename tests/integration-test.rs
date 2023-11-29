use sslb::lb::LoadBalancer;
use sslb::policy::{SimpleRoundRobinPolicy, Policy};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const HTTP_OK_RESPONSE: &'static str = "HTTP/1.1 200 OK\r\n";
const HTTP_GET_REQUEST: &'static str = "GET / HTTP/1.1\r\n";

async fn create_dummy_endpoint(addr: &str) {
  println!("Create dummy endpoint with addr: {}", addr);
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

async fn create_lb_with_policy(addr: &str, endpoints: Vec<String>, policy: Box<dyn Policy<String> + Send>) {
  let mut server = LoadBalancer::build(addr, endpoints, policy).await.unwrap();
  tokio::spawn(async move {
    server.run().await;
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
  let policy = Box::new(SimpleRoundRobinPolicy::new());
  create_lb_with_policy(server_addr, endpoints, policy).await;

  let mut stream = TcpStream::connect(server_addr).await.unwrap();
  stream.write_all(HTTP_GET_REQUEST.as_bytes()).await.unwrap();
  let mut buf = vec![0u8; 1024];
  let _bytes_read = stream.read(&mut buf).await.unwrap();
  assert_eq!(HTTP_OK_RESPONSE, std::str::from_utf8(&buf).unwrap().trim_end_matches("\0"));
}
