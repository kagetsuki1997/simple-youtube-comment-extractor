pub const HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

Options:
{options}

Commands:
{subcommands}

{after-help}";

#[allow(dead_code)]
pub const LOG_PATH: &str = "./logs";

pub const YOUTUBE_COMMENT_THREADS_API:&str = "https://www.googleapis.com/youtube/v3/commentThreads?part=snippet%2Creplies&maxResults=100&order=time";
