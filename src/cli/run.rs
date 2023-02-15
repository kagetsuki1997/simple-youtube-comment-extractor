use clap::Parser;
use reqwest::Url;
use serde_json::Value;
use snafu::{ResultExt, Snafu};
use xlsxwriter::{Workbook, Worksheet};

use crate::{
    consts, error,
    error::{CommandError, Result},
};

#[derive(Debug, Snafu)]
pub enum RunError {
    #[snafu(display(
        "Error occurs while creating `xlsxwriter::Workbook` in `./output.xlsx`, error: {source}"
    ))]
    CreateXlsxWorkbook { source: xlsxwriter::XlsxError },

    #[snafu(display(
        "Error occurs while closing `xlsxwriter::Workbook` in `./output.xlsx`, error: {source}"
    ))]
    CloseXlsxWorkbook { source: xlsxwriter::XlsxError },

    #[snafu(display("Error occurs while adding Worksheet `{name}`, error: {source}"))]
    AddXlsxWorksheet { name: String, source: xlsxwriter::XlsxError },

    #[snafu(display("Error occurs while writing column, error: {source}"))]
    WriteXlsxColumn { source: xlsxwriter::XlsxError },

    #[snafu(display("Error occurs while building `reqwest::Client`, error: {source}"))]
    BuildReqwestClient { source: reqwest::Error },

    #[snafu(display("Error occurs while parsing url \"{url}\".\nError: {source}"))]
    ParseUrl { url: String, source: url::ParseError },

    #[snafu(display("Error occurs while doing GET request.\nError: {source}"))]
    GetRequest { source: reqwest::Error },

    #[snafu(display("Error occurs while deserializing response body.\nError: {source}"))]
    DeserializingResponseBody { source: reqwest::Error },
}

impl CommandError for RunError {
    fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::CreateXlsxWorkbook { .. }
            | Self::CloseXlsxWorkbook { .. }
            | Self::AddXlsxWorksheet { .. }
            | Self::WriteXlsxColumn { .. } => exitcode::IOERR,
            Self::ParseUrl { .. } => exitcode::DATAERR,
            Self::BuildReqwestClient { .. }
            | Self::GetRequest { .. }
            | Self::DeserializingResponseBody { .. } => exitcode::SOFTWARE,
        }
    }
}

#[derive(Debug, Parser)]
#[command(help_template(consts::HELP_TEMPLATE), disable_help_subcommand(true))]
pub struct Command {
    #[arg(long, help = "API Key")]
    api_key: String,

    #[arg(name = "video-ids", help = "Video IDs")]
    video_ids: Vec<String>,
}

impl Command {
    pub fn run(self) -> Result<()> {
        let headers: Vec<&str> = vec![
            "etag",
            "author_display_name",
            "author_channel_url",
            "text_display",
            "published_at",
            "updated_at",
            "replied_etag",
            "replied_author_display_name",
            "replied_author_channel_url",
            "replied_text_display",
            "replied_published_at",
            "replied_updated_at",
        ];

        tokio::runtime::Runtime::new().context(error::InitializeTokioRuntimeSnafu)?.block_on(
            async move {
                let workbook = Workbook::new("output.xlsx").context(CreateXlsxWorkbookSnafu)?;

                let client = reqwest::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .context(BuildReqwestClientSnafu)?;

                for video_id in self.video_ids {
                    println!("Start to extract comments from {video_id}");

                    // 1. create api url
                    let url = Url::parse_with_params(
                        consts::YOUTUBE_COMMENT_THREADS_API,
                        &[("key", self.api_key.clone()), ("videoId", video_id.clone())],
                    )
                    .context(ParseUrlSnafu {
                        url: consts::YOUTUBE_COMMENT_THREADS_API.to_string(),
                    })?;

                    // 2. try to get first page
                    let mut body: Value = client
                        .get(url.clone())
                        .send()
                        .await
                        .context(GetRequestSnafu)?
                        .json()
                        .await
                        .context(DeserializingResponseBodySnafu)?;

                    // 2.1 create sheet
                    let mut sheet = workbook
                        .add_worksheet(Some(video_id.as_str()))
                        .context(AddXlsxWorksheetSnafu { name: video_id })?;

                    // 2.2 write header
                    for (i, header) in headers.iter().enumerate() {
                        sheet
                            .write_string(
                                0,
                                i.try_into().expect("the header size must in u16"),
                                header,
                                None,
                            )
                            .context(WriteXlsxColumnSnafu)?;
                    }

                    // 2.3 write rows
                    let mut row = write_comment_rows(&mut sheet, 1, &body)?;

                    // 3. try to get all pages
                    loop {
                        let next_page_token = body["nextPageToken"].clone();
                        if next_page_token.is_null() {
                            break;
                        }

                        let url = Url::parse_with_params(
                            url.as_str(),
                            &[("pageToken", next_page_token.as_str().unwrap_or(""))],
                        )
                        .context(ParseUrlSnafu { url: url.to_string() })?;

                        body = client
                            .get(url)
                            .send()
                            .await
                            .context(GetRequestSnafu)?
                            .json()
                            .await
                            .context(DeserializingResponseBodySnafu)?;

                        // 2.3 write rows
                        row = write_comment_rows(&mut sheet, row, &body)?;
                    }
                }

                workbook.close().context(CloseXlsxWorkbookSnafu)?;
                println!("Success to save result to `output.xlsx` !");
                Ok(())
            },
        )
    }
}

fn write_comment_rows(sheet: &mut Worksheet, start_at: u32, body: &Value) -> Result<u32> {
    let mut row = start_at;
    if let Some(items) = body["items"].as_array() {
        for item in items {
            sheet
                .write_string(
                    row,
                    0,
                    item["snippet"]["topLevelComment"]["etag"].as_str().unwrap_or(""),
                    None,
                )
                .context(WriteXlsxColumnSnafu)?;
            sheet
                .write_string(
                    row,
                    1,
                    item["snippet"]["topLevelComment"]["snippet"]["authorDisplayName"]
                        .as_str()
                        .unwrap_or(""),
                    None,
                )
                .context(WriteXlsxColumnSnafu)?;
            sheet
                .write_string(
                    row,
                    2,
                    item["snippet"]["topLevelComment"]["snippet"]["authorChannelUrl"]
                        .as_str()
                        .unwrap_or(""),
                    None,
                )
                .context(WriteXlsxColumnSnafu)?;
            sheet
                .write_string(
                    row,
                    3,
                    item["snippet"]["topLevelComment"]["snippet"]["textDisplay"]
                        .as_str()
                        .unwrap_or(""),
                    None,
                )
                .context(WriteXlsxColumnSnafu)?;
            sheet
                .write_string(
                    row,
                    4,
                    item["snippet"]["topLevelComment"]["snippet"]["publishedAt"]
                        .as_str()
                        .unwrap_or(""),
                    None,
                )
                .context(WriteXlsxColumnSnafu)?;
            sheet
                .write_string(
                    row,
                    5,
                    item["snippet"]["topLevelComment"]["snippet"]["updatedAt"]
                        .as_str()
                        .unwrap_or(""),
                    None,
                )
                .context(WriteXlsxColumnSnafu)?;

            row += 1;

            // write replies
            if let Some(replies) = item["replies"]["comments"].as_array() {
                for reply in replies {
                    sheet
                        .write_string(row, 6, reply["snippet"]["etag"].as_str().unwrap_or(""), None)
                        .context(WriteXlsxColumnSnafu)?;
                    sheet
                        .write_string(
                            row,
                            7,
                            reply["snippet"]["authorDisplayName"].as_str().unwrap_or(""),
                            None,
                        )
                        .context(WriteXlsxColumnSnafu)?;
                    sheet
                        .write_string(
                            row,
                            8,
                            reply["snippet"]["authorChannelUrl"].as_str().unwrap_or(""),
                            None,
                        )
                        .context(WriteXlsxColumnSnafu)?;
                    sheet
                        .write_string(
                            row,
                            9,
                            reply["snippet"]["textDisplay"].as_str().unwrap_or(""),
                            None,
                        )
                        .context(WriteXlsxColumnSnafu)?;
                    sheet
                        .write_string(
                            row,
                            10,
                            reply["snippet"]["publishedAt"].as_str().unwrap_or(""),
                            None,
                        )
                        .context(WriteXlsxColumnSnafu)?;
                    sheet
                        .write_string(
                            row,
                            11,
                            reply["snippet"]["updatedAt"].as_str().unwrap_or(""),
                            None,
                        )
                        .context(WriteXlsxColumnSnafu)?;

                    row += 1;
                }
            }
        }
    }

    Ok(row)
}
