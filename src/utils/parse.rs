use log::warn;

const DEFAULT_TIME_SECONDS: u64 = 86400;

pub fn string_to_time_second(s: &str) -> u64 {
    // convert string to lower case
    let s = s.to_lowercase();

    // if string ends with 's', remove it and convert to integer
    if s.ends_with("s") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value,
            Err(_) => {
                warn!(
                    "Failed to parse seconds from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'm', remove it and convert to integer
    else if s.ends_with("m") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60,
            Err(_) => {
                warn!(
                    "Failed to parse minutes from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'h', remove it and convert to integer
    else if s.ends_with("h") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60,
            Err(_) => {
                warn!(
                    "Failed to parse hours from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'd', remove it and convert to integer
    else if s.ends_with("d") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60 * 24,
            Err(_) => {
                warn!(
                    "Failed to parse days from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'w', remove it and convert to integer
    else if s.ends_with("w") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60 * 24 * 7,
            Err(_) => {
                warn!(
                    "Failed to parse weeks from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'y', remove it and convert to integer
    else if s.ends_with("y") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60 * 24 * 365,
            Err(_) => {
                warn!(
                    "Failed to parse years from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if no suffix, just parse as a plain integer
    else {
        match s.parse::<u64>() {
            Ok(value) => value,
            Err(_) => {
                warn!(
                    "Failed to parse integer from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
}
