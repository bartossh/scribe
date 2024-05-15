use rand::rngs::ThreadRng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use ureq;

const WAIT_MS: u64 = 5;
const ROUNDS: usize = 1000;

#[derive(Debug, Serialize, Deserialize)]
struct LogInput {
    log: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Query {
    prefix: Option<String>,
    words: Option<Vec<String>>,
    from: u64,
    to: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogsOutput {
    logs: Vec<String>,
}

fn file_read_helper() -> Result<Vec<String>> {
    let file = File::open("assets/quotes.txt")?;
    let reader = BufReader::new(file);
    let mut result = Vec::new();

    for line in reader.lines() {
        result.push(line?);
    }

    Ok(result)
}

fn mix_and_merge(rng: &mut ThreadRng, rounds: usize, data: &[String]) -> String {
    let mut result: String = "".to_string();
    for _ in 0..rounds {
        let idx = rng.gen_range(0..data.len());
        result.push_str(&data[idx])
    }

    result
}

#[test]
#[ignore]
fn integration_bench_create_log() -> Result<()> {
    let path = "http://0.0.0.0:8000/save";

    let mut rng = rand::thread_rng();
    let logs = file_read_helper()?;

    let start = Instant::now();
    for _ in 0..ROUNDS {
        let log = mix_and_merge(&mut rng, 5, &logs);
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: log });
        match status {
            Ok(resp) => assert_eq!(resp.status(), 200),
            Err(e) => {
                println!("err {}", e);
                assert!(false);
            }
        };
        sleep(Duration::from_millis(WAIT_MS));
    }

    let duration = start.elapsed();

    println!(
        "create_log test took per request [ {:?} ms ], total [ {:?} ms ] for {} request",
        (duration.as_millis() as u32 - (ROUNDS as u64 * WAIT_MS) as u32) / ROUNDS as u32,
        duration.as_millis() as u32 - (ROUNDS as u64 * WAIT_MS) as u32,
        ROUNDS
    );

    Ok(())
}
#[test]
#[ignore]
fn integration_bench_read_log() -> Result<()> {
    let path = "http://0.0.0.0:8000/save";

    let mut rng = rand::thread_rng();
    let logs = file_read_helper()?;

    let time_from = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    for _ in 0..ROUNDS {
        let log = mix_and_merge(&mut rng, 5, &logs);
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: log });
        match status {
            Ok(resp) => assert_eq!(resp.status(), 200),
            Err(e) => {
                println!("err {}", e);
                assert!(false);
            }
        };
        sleep(Duration::from_millis(WAIT_MS));
    }

    let time_to = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    sleep(Duration::from_millis(100));

    let path = "http://0.0.0.0:8000/read";

    let start = Instant::now();
    for _ in 0..ROUNDS {
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&Query {
                prefix: Some("th".to_string()),
                words: Some(vec![
                    "looking".to_string(),
                    "not".to_string(),
                    "focusing".to_string(),
                    "stop".to_string(),
                ]),
                from: time_from.as_nanos() as u64,
                to: time_to.as_nanos() as u64,
            });
        match status {
            Ok(resp) => assert_eq!(resp.status(), 200),
            Err(e) => {
                println!("err {}", e);
                assert!(false);
            }
        };
        sleep(Duration::from_millis(WAIT_MS));
    }

    let duration = start.elapsed();

    println!(
        "read_log test took per request [ {:?} ms ], total [ {:?} ms ] for {} request",
        (duration.as_millis() as u32 - (ROUNDS as u64 * WAIT_MS) as u32) / ROUNDS as u32,
        duration.as_millis() as u32 - (ROUNDS as u64 * WAIT_MS) as u32,
        ROUNDS
    );

    Ok(())
}
