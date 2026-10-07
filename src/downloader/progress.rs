use indicatif::{ProgressBar, ProgressStyle};

pub fn create_download_progress_bar(total_bytes: Option<u64>, prefix: &str) -> ProgressBar {
    let pb = match total_bytes {
        Some(total) => {
            let pb = ProgressBar::new(total);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{prefix:.bold} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("━╸ "),
            );
            pb
        }
        None => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{prefix:.bold} [{elapsed_precise}] {bytes} downloaded... {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner()),
            );
            pb
        }
    };
    pb.set_prefix(prefix.to_string());
    pb
}
