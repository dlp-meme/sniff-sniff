use reqwest::Client;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

type SharedFile = Arc<Mutex<std::fs::File>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Настраиваем файл для логирования запросов
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("requests.log")
        .expect("Unable to open log file");

    let log_file = Arc::new(Mutex::new(log_file));

    // Настроим адрес сервера
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(&addr).await?;

    println!("Proxy server running at http://{}", addr);

    // Обрабатываем входящие соединения
    loop {
        let (mut socket, _) = listener.accept().await?;

        let log_file = log_file.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(&mut socket, log_file).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_connection(socket: &mut tokio::net::TcpStream, log_file: SharedFile) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = vec![0u8; 4096];
    let bytes_read = socket.read(&mut buffer).await?;

    if bytes_read == 0 {
        return Ok(());
    }

    // Преобразуем запрос в строку
    let request_data = String::from_utf8_lossy(&buffer[..bytes_read]);

    // Логируем запрос
    log_request(&request_data, log_file).await;

    // Извлекаем строку HTTP-запроса
    let request_lines: Vec<&str> = request_data.lines().collect();
    let first_line = request_lines.get(0).map_or("", |line| line);
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let method = parts.get(0).map_or("GET", |method| method);
    let uri = parts.get(1).map_or("/", |uri| uri);

    // Перенаправляем запрос на целевой сервер
    let client = Client::new();
    let response = client
        .request(method.parse()?, uri)
        .send()
        .await?;

    // Отправляем ответ клиенту
    let mut response_data = Vec::new();
    response_data.push(b"HTTP/1.1 200 OK\r\n".to_vec());
    response_data.push(format!("Content-Length: {}\r\n", response.content_length().unwrap_or(0)).into_bytes());
    response_data.push(b"\r\n".to_vec());
    response_data.push(response.bytes().await?.to_vec());

    socket.write_all(&response_data.concat()).await?;

    Ok(())
}

// Функция логирования запроса
async fn log_request(req: &str, log_file: SharedFile) {
    let log_entry = format!("--- Request ---\n{}\n\n", req);

    let mut file = log_file.lock().await;
    if let Err(e) = file.write_all(log_entry.as_bytes()) {
        eprintln!("Failed to write to log file: {}", e);
    }
}
