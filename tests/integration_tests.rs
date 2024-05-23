use rand::rngs::ThreadRng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{Error, Result};
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use ureq;

const WAIT_MS: u64 = 2;
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
fn on_create_log_api_call_should_respond_with_code_200() -> Result<()> {
    let path = "http://localhost:8000/save";

    let status = ureq::post(&path)
        .set("Content-Type", "application/json")
        .send_json(&LogInput {
            log: "I am the log that is added in to application".to_string(),
        });
    match status {
        Ok(resp) => assert_eq!(resp.status(), 200),
        Err(e) => {
            println!("err {}", e);
            assert!(false);
        }
    };
    sleep(Duration::from_millis(WAIT_MS));
    Ok(())
}

#[test]
#[ignore]
fn on_read_log_api_call_match_words_and_prefix_should_use_multiple_query_params_and_respond_with_logs_matching_all_query_params(
) -> Result<()> {
    let path = "http://localhost:8000/save";

    let time_from = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    for l in vec![
        "This log is number one prefix match",
        "This log is number two",
        "This log is number three prefix match",
        "This log is number four",
        "Not found one",
        "Not found two but prefix match",
        "Not found three",
        "Not found four but prefix match",
        "Not found five",
    ] {
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: l.to_string() });
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

    sleep(Duration::from_millis(10));

    for l in vec![
        "Outside of time range. This log is number one prefix match",
        "Outside of time range. This log is number two prefix match",
        "Outside of time range. This log is number three prefix match",
        "Outside of time range. This log is number four prefix match",
    ] {
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: l.to_string() });
        match status {
            Ok(resp) => assert_eq!(resp.status(), 200),
            Err(e) => {
                println!("err {}", e);
                assert!(false);
            }
        };
        sleep(Duration::from_millis(WAIT_MS));
    }

    let path = "http://localhost:8000/read";

    let status = ureq::post(&path)
        .set("Content-Type", "application/json")
        .send_json(&Query {
            prefix: Some("pref".to_string()),
            words: Some(vec![
                "This".to_string(),
                "log".to_string(),
                "is".to_string(),
                "number".to_string(),
            ]),
            from: time_from.as_nanos() as u64,
            to: time_to.as_nanos() as u64,
        });
    sleep(Duration::from_millis(10));

    match status {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let logs: LogsOutput = resp.into_json()?;
            assert_eq!(logs.logs.len(), 2);
            for w in "This log is number".split_whitespace() {
                for log in logs.logs.iter() {
                    let sl: Vec<String> = log.split_whitespace().map(str::to_string).collect();
                    let Some(_) = sl.iter().find(|s| *s == w) else {
                        assert!(false);
                        return Err(Error::new(std::io::ErrorKind::NotFound, "item not found"));
                    };
                }
            }
        }
        Err(e) => {
            println!("err {}", e);
            assert!(false);
        }
    };

    Ok(())
}

#[test]
#[ignore]
fn on_read_log_api_call_match_existing_word_should_respond_with_previously_added_log_that_match_the_query(
) -> Result<()> {
    let path = "http://localhost:8000/save";

    let time_from = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    for l in vec![
        "This log is number one",
        "This log is number two",
        "This log is number three",
        "This log is number four",
        "Not found one",
        "Not found two",
        "Not found three",
        "Not found four",
        "Not found five",
    ] {
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: l.to_string() });
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

    let path = "http://localhost:8000/read";

    let status = ureq::post(&path)
        .set("Content-Type", "application/json")
        .send_json(&Query {
            prefix: None,
            words: Some(vec![
                "This".to_string(),
                "log".to_string(),
                "is".to_string(),
                "number".to_string(),
            ]),
            from: time_from.as_nanos() as u64,
            to: time_to.as_nanos() as u64,
        });
    match status {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let logs: LogsOutput = resp.into_json()?;
            assert_eq!(logs.logs.len(), 4);
            for w in "This log is number".split_whitespace() {
                for log in logs.logs.iter() {
                    let sl: Vec<String> = log.split_whitespace().map(str::to_string).collect();
                    let Some(_) = sl.iter().find(|s| *s == w) else {
                        assert!(false);
                        return Err(Error::new(std::io::ErrorKind::NotFound, "item not found"));
                    };
                }
            }
        }
        Err(e) => {
            println!("err {}", e);
            assert!(false);
        }
    };

    Ok(())
}

#[test]
#[ignore]
fn on_read_log_api_call_match_existing_word_should_respond_with_empty_message_when_logs_do_not_match(
) -> Result<()> {
    let path = "http://localhost:8000/save";

    let time_from = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    for l in vec![
        "Not found one",
        "Not found two",
        "Not found three",
        "Not found four",
        "Not found five",
    ] {
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: l.to_string() });
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

    let path = "http://localhost:8000/read";

    let status = ureq::post(&path)
        .set("Content-Type", "application/json")
        .send_json(&Query {
            prefix: None,
            words: Some(vec![
                "This".to_string(),
                "log".to_string(),
                "is".to_string(),
                "number".to_string(),
            ]),
            from: time_from.as_nanos() as u64,
            to: time_to.as_nanos() as u64,
        });
    match status {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let logs: LogsOutput = resp.into_json()?;
            assert_eq!(logs.logs.len(), 0);
        }
        Err(e) => {
            println!("err {}", e);
            assert!(false);
        }
    };

    Ok(())
}

#[test]
#[ignore]
fn on_read_log_api_call_match_word_prefix_should_respond_with_messages_when_logs_has_matching_prefix(
) -> Result<()> {
    let path = "http://localhost:8000/save";

    let time_from = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    for l in vec![
        "This log is super funny should found",
        "This log is super fun should found",
        "This log is super fan",
        "This log is super fantastic",
        "This log is super fonetic",
        "This log is super fixed",
        "This log is super fu should found",
    ] {
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: l.to_string() });
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

    let path = "http://localhost:8000/read";

    let status = ureq::post(&path)
        .set("Content-Type", "application/json")
        .send_json(&Query {
            prefix: Some("fu".to_string()),
            words: None,
            from: time_from.as_nanos() as u64,
            to: time_to.as_nanos() as u64,
        });
    match status {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let logs: LogsOutput = resp.into_json()?;
            assert_eq!(logs.logs.len(), 3);
            for w in "should found".split_whitespace() {
                for log in logs.logs.iter() {
                    let sl: Vec<String> = log.split_whitespace().map(str::to_string).collect();
                    let Some(_) = sl.iter().find(|s| *s == w) else {
                        assert!(false);
                        return Err(Error::new(std::io::ErrorKind::NotFound, "item not found"));
                    };
                }
            }
        }
        Err(e) => {
            println!("err {}", e);
            assert!(false);
        }
    };

    Ok(())
}

#[test]
#[ignore]
fn on_read_log_api_call_match_word_prefix_should_respond_with_empty_response_when_prefix_is_not_matching(
) -> Result<()> {
    let path = "http://localhost:8000/save";

    let time_from = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    for l in vec![
        "This log is super funny should found",
        "This log is super fun should found",
        "This log is super fan",
        "This log is super fantastic",
        "This log is super fonetic",
        "This log is super fixed",
        "This log is super fu should found",
    ] {
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: l.to_string() });
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

    let path = "http://localhost:8000/read";

    let status = ureq::post(&path)
        .set("Content-Type", "application/json")
        .send_json(&Query {
            prefix: Some("xx".to_string()),
            words: None,
            from: time_from.as_nanos() as u64,
            to: time_to.as_nanos() as u64,
        });
    match status {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let logs: LogsOutput = resp.into_json()?;
            assert_eq!(logs.logs.len(), 0);
        }
        Err(e) => {
            println!("err {}", e);
            assert!(false);
        }
    };

    Ok(())
}

#[test]
#[ignore]
fn on_read_log_api_call_match_time_should_find_all_in_time_range() -> Result<()> {
    let path = "http://localhost:8000/save";

    let time_from = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    for l in vec![
        "This log is super funny should found",
        "This log is super fun should found",
        "This log is super fan should found",
        "This log is super fantastic should found",
        "This log is super fonetic should found",
        "This log is super fixed should found",
        "This log is super fu should found",
    ] {
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: l.to_string() });
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

    let path = "http://localhost:8000/read";

    let status = ureq::post(&path)
        .set("Content-Type", "application/json")
        .send_json(&Query {
            prefix: None,
            words: None,
            from: time_from.as_nanos() as u64,
            to: time_to.as_nanos() as u64,
        });
    match status {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let logs: LogsOutput = resp.into_json()?;
            assert_eq!(logs.logs.len(), 7);
            for w in "should found".split_whitespace() {
                for log in logs.logs.iter() {
                    let sl: Vec<String> = log.split_whitespace().map(str::to_string).collect();
                    let Some(_) = sl.iter().find(|s| *s == w) else {
                        assert!(false);
                        return Err(Error::new(std::io::ErrorKind::NotFound, "item not found"));
                    };
                }
            }
        }
        Err(e) => {
            println!("err {}", e);
            assert!(false);
        }
    };

    Ok(())
}

#[test]
#[ignore]
fn on_read_log_api_call_match_should_return_empty_result_for_time_rang_with_no_matching_logs(
) -> Result<()> {
    let path = "http://localhost:8000/save";

    let time_from = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    sleep(Duration::from_millis(10));
    let time_to = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    sleep(Duration::from_millis(10));
    for l in vec![
        "This log is super funny",
        "This log is super fun",
        "This log is super fan",
        "This log is super fantastic",
        "This log is super fonetic",
        "This log is super fixed",
        "This log is super fu",
    ] {
        let status = ureq::post(&path)
            .set("Content-Type", "application/json")
            .send_json(&LogInput { log: l.to_string() });
        match status {
            Ok(resp) => assert_eq!(resp.status(), 200),
            Err(e) => {
                println!("err {}", e);
                assert!(false);
            }
        };
        sleep(Duration::from_millis(WAIT_MS));
    }

    sleep(Duration::from_millis(100));

    let path = "http://localhost:8000/read";

    let status = ureq::post(&path)
        .set("Content-Type", "application/json")
        .send_json(&Query {
            prefix: None,
            words: None,
            from: time_from.as_nanos() as u64,
            to: time_to.as_nanos() as u64,
        });
    match status {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let logs: LogsOutput = resp.into_json()?;
            assert_eq!(logs.logs.len(), 0);
        }
        Err(e) => {
            println!("err {}", e);
            assert!(false);
        }
    };

    Ok(())
}

#[test]
#[ignore]
fn integration_bench_create_log() -> Result<()> {
    let path = "http://localhost:8000/save";

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
    let path = "http://localhost:8000/save";

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

    let path = "http://localhost:8000/read";

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
                    "fail".to_string(),
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
