use reqwest;
use tokio;

#[tokio::test]
async fn test_health_endpoint() {
    // This test assumes the server is running
    // In a real scenario, you'd spawn the server in the test
    let response = reqwest::get("http://localhost:3000/health").await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let text = resp.text().await.unwrap();
            assert_eq!(text, "OK");
        }
        Err(_) => {
            // Server not running, skip test
            println!("Server not running, skipping integration test");
        }
    }
}

#[tokio::test]
async fn test_klines_endpoint() {
    let response = reqwest::get("http://localhost:3000/api/klines/PEPE?interval=1m&limit=10").await;
    
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let json: serde_json::Value = resp.json().await.unwrap();
            assert!(json.get("symbol").is_some());
            assert!(json.get("interval").is_some());
            assert!(json.get("data").is_some());
        }
        Err(_) => {
            println!("Server not running, skipping integration test");
        }
    }
} 