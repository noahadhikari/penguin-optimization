use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use reqwest;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct APIResponse {
	Entries: Vec<Scores>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct Scores {
	TeamName:  String,
	TeamScore: f64,
}

pub enum InputType {
	Small,
	Medium,
	Large,
}

/// Prints out the inputs we have better/worse scores than
#[tokio::main]
pub async fn get_api_result(size: InputType) {
	let input_type: &str;
	// { test_number: [our_score, leaderboard_score], ... }
	let mut worse_scores: BTreeMap<u8, Vec<f64>> = BTreeMap::new();
	let mut better_scores: BTreeMap<u8, Vec<f64>> = BTreeMap::new();

	// Maps to directory name
	match size {
		InputType::Small => input_type = "small",
		InputType::Medium => input_type = "medium",
		InputType::Large => input_type = "large",
	}

	// Number of tests in each size
	let input_count: BTreeMap<&str, u8> = BTreeMap::from([("small", 241), ("medium", 239), ("large", 239)]);

	let count = *input_count.get(input_type).unwrap();
	for i in 1..=count {
		if i == 240 && input_type == "small" {
			// small/240 is invalid
			continue;
		}

		let highest_score = get_best_leaderboard_score(i, &input_type).await;
		match highest_score {
			Err(e) => panic!("{}", e),
			Ok(leaderboard_penalty) => {
				// Found highest leaderboard score
				println!("{}: {:?}", i, leaderboard_penalty);
				let our_path = "./outputs/".to_string() + &input_type.to_string() + "/" + &get_three_digit_num(i) + ".out";
				// We don't have an output file
				if !Path::new(&our_path).is_file() {
					println!("Local test {} not found", i.to_string());
					continue;
				}

				let our_penalty = round(get_penalty_from_file(our_path.as_str()));
				let rounded_leaderboard = round(leaderboard_penalty);

				if our_penalty > rounded_leaderboard {
					worse_scores.insert(i, vec![our_penalty, rounded_leaderboard]);
				} else if our_penalty < rounded_leaderboard {
					better_scores.insert(i, vec![our_penalty, rounded_leaderboard]);
				}
			}
		}
	}

	println!("\n\n\n\n");
	println!("{} Better:", better_scores.len());
	for (key, value) in better_scores {
		println!("Test {}. Ours: {}. Best: {}. Diff: {}", key, value[0], value[1], round(value[1] - value[0]));
	}

	println!("\n{} Worse:", worse_scores.len());
	for (key, value) in worse_scores {
		println!("Test {}. Ours: {}. Best: {}. Diff: {}", key, value[0], value[1], round(value[0] - value[1]));
	}
}

/// Rounds number to 6 decimal places to avoid floating point errors
fn round(n: f64) -> f64 {
	(n * 1000000.0).round() / 1000000.0
}

/// Converts number to 3 digit equivalent (1 -> "001", 40 -> "040", 103 ->
/// "103")
fn get_three_digit_num(n: u8) -> String {
	if n >= 100 {
		return n.to_string();
	} else if n >= 10 {
		return "0".to_string() + &n.to_string();
	} else {
		return "00".to_string() + &n.to_string();
	}
}

/// Gets our penalty from a specifie file
pub fn get_penalty_from_file(path: &str) -> f64 {
	let file = File::open(path).unwrap();
	let reader = BufReader::new(file);
	let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
	let penalty_line = lines.get(0).unwrap(); // Penalty = xxx
	let split_line: Vec<&str> = penalty_line.split_whitespace().collect();
	let existing_penalty: f64 = split_line.get(3).unwrap().parse::<f64>().unwrap();
	existing_penalty
}

/// Returns the best leaderboard score for the given test case
async fn get_best_leaderboard_score(test_num: u8, input_type: &str) -> Result<f64, String> {
	let get_url = "https://project.cs170.dev/scoreboard/".to_string() + input_type + "/" + &test_num.to_string();

	let res = reqwest::get(get_url).await.unwrap();

	match res.status() {
		reqwest::StatusCode::OK => {
			match res.json::<APIResponse>().await {
				Ok(parsed) => {
					return Ok(get_min_score(parsed.Entries));
				}
				Err(_) => return Err("The response didn't match the shape we expected.".to_string()),
			};
		}
		other => return Err("Other error occurred".to_string() + other.as_str()),
	}
}

/// Returns the minimum score of a vector of scores
fn get_min_score(scores: Vec<Scores>) -> f64 {
	let mut cur_min = f64::MAX;
	for score in scores {
		cur_min = cur_min.min(score.TeamScore);
	}
	cur_min
}
