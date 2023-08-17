use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionFunctionCall, ChatCompletionFunctions, ChatCompletionRequestMessage,
        CreateChatCompletionRequestArgs, FunctionCall, Role,
    },
};
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{error, info};
use question::{Answer, Question};
use rand::seq::SliceRandom;
use schemars::{
    gen::{SchemaGenerator, SchemaSettings},
    JsonSchema,
};
use serde_json::json;
use spinners::{Spinner, Spinners};
use std::{
    io::Write,
    process::{Command, Stdio},
    str,
};
use std::env;
#[derive(Parser)]
#[command(version)]
#[command(name = "Auto Commit")]
#[command(author = "Miguel Piedrafita <soy@miguelpiedrafita.com>")]
#[command(about = "Automagically generate commit messages. 202308, 适配国内使用", long_about = None)]
struct Cli {
    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,

    #[arg(
        long = "dry-run",
        help = "输出生成的消息，但不创建提交。"
    )]
    dry_run: bool,

    #[arg(
        short,
        long,
        help = "在提交之前编辑生成的提交消息。"
    )]
    review: bool,

    #[arg(short, long, help = "在提交之前不要询问确认。")]
    force: bool,
}


#[derive(Debug, serde::Deserialize, JsonSchema)]
struct Commit {
    /// The title of the commit.
    title: String,

    /// An exhaustive description of the changes.
    description: String,
}

impl ToString for Commit {
    fn to_string(&self) -> String {
        format!("{}\n\n{}", self.title, self.description)
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    env::set_var("OPENAI_API_BASE", "https://ai.fakeopen.com");

    let api_token = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        //自己编译使用的话，可以把API_KEY写死在这里
        env::set_var("OPENAI_API_KEY", "sk-NOT-SET");
    });


    let git_staged_cmd = Command::new("git")
        .arg("diff")
        .arg("--staged")
        .output()
        .expect("找不到命令 diff.")
        .stdout;

    let git_staged_cmd = str::from_utf8(&git_staged_cmd).unwrap();

    if git_staged_cmd.is_empty() {
        error!("没有要提交的暂存文件。\n尝试运行 `git add` 命令来暂存一些文件。");
    }

    let is_repo = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .expect("无法检查是否为 Git 仓库。")
        .stdout;

    if str::from_utf8(&is_repo).unwrap().trim() != "true" {
        error!("看起来你不在一个 Git 仓库中。\n请从 Git 仓库的根目录运行此命令，或者使用 `git init` 初始化一个新的仓库。");
        std::process::exit(1);
    }


     let client = async_openai::Client::with_config(OpenAIConfig::new().with_api_key(api_token));
     //let client = async_openai::Client::with_config(OpenAIConfig::new().with_api_key(api_token).with_base_url(base_url));


    let output = Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .output()
        .expect("Couldn't find diff.")
        .stdout;
    let output = str::from_utf8(&output).unwrap();

    if !cli.dry_run {
        info!("Loading Data...");
    }

    let sp: Option<Spinner> = if !cli.dry_run && cli.verbose.is_silent() {
        let vs = [
            Spinners::Earth,
            Spinners::Aesthetic,
            Spinners::Hearts,
            Spinners::BoxBounce,
            Spinners::BoxBounce2,
            Spinners::BouncingBar,
            Spinners::Christmas,
            Spinners::Clock,
            Spinners::FingerDance,
            Spinners::FistBump,
            Spinners::Flip,
            Spinners::Layer,
            Spinners::Line,
            Spinners::Material,
            Spinners::Mindblown,
            Spinners::Monkey,
            Spinners::Noise,
            Spinners::Point,
            Spinners::Pong,
            Spinners::Runner,
            Spinners::SoccerHeader,
            Spinners::Speaker,
            Spinners::SquareCorners,
            Spinners::Triangle,
        ];

        let spinner = vs.choose(&mut rand::thread_rng()).unwrap().clone();

        Some(Spinner::new(spinner, "分析代码库...".into()))
    } else {
        None
    };

    let mut generator = SchemaGenerator::new(SchemaSettings::openapi3().with(|settings| {
        settings.inline_subschemas = true;
    }));

    let commit_schema = generator.subschema_for::<Commit>().into_object();

    let completion = client
        .chat()
        .create(
            CreateChatCompletionRequestArgs::default()
                .messages(vec![
                    ChatCompletionRequestMessage {
                        role: Role::System,
                        content: Some(
                           // "You are an experienced programmer who writes great commit messages in CHINESE."
                           "你是一个经验丰富的程序开发人员，提交代码时会清晰准确的书写commit信息。所有的commit信息都用中文输出。"
                                .to_string(),
                        ),
                        ..Default::default()
                    },
                    ChatCompletionRequestMessage {
                        role: Role::Assistant,
                        content: Some("".to_string()),
                        function_call: Some(FunctionCall {
                            arguments: "{}".to_string(),
                            name: "get_diff".to_string(),
                        }),
                        ..Default::default()
                    },
                    ChatCompletionRequestMessage {
                        role: Role::Function,
                        content: Some(output.to_string()),
                        name: Some("get_diff".to_string()),
                        ..Default::default()
                    },
                ])
                .functions(vec![
                    ChatCompletionFunctions {
                        name: "get_diff".to_string(),
                        description: Some(
                            "返回 `git status` 的结果为字符串，并翻译为中文".to_string(),
                        ),
                        parameters: Some(json!({
                            "type": "object",
                            "properties": {}
                        })),
                    },
                    ChatCompletionFunctions {
                        name: "commit".to_string(),
                        description: Some(
                           // "Creates a commit with the given title and a description.".to_string(),
                           "根据提供的描述，创建一个简洁的commit. 并用中文输出".to_string(),
                        ),
                        parameters: Some(serde_json::to_value(commit_schema).unwrap()),
                    },
                ])
                .function_call(ChatCompletionFunctionCall::Object(
                    json!({ "name": "commit" }),
                ))
                .model("gpt-3.5-turbo-16k")
                .temperature(0.1)
                .max_tokens(2000u16)
                .build()
                .unwrap(),
        )
        .await
        .expect("Couldn't complete prompt.");

    if sp.is_some() {
        sp.unwrap().stop_with_message("Finished Analyzing!".into());
    }

    let commit_data = &completion.choices[0].message.function_call;
    let commit_msg = serde_json::from_str::<Commit>(&commit_data.as_ref().unwrap().arguments)
        .expect("Couldn't parse model response.")
        .to_string();

    if cli.dry_run {
        info!("{}", commit_msg);
        return Ok(());
    } else {
        info!(
            "Proposed Commit:\n------------------------------\n{}\n------------------------------",
            commit_msg
        );

        if !cli.force {
            let answer = Question::new("Do you want to continue? (Y/n)")
                .yes_no()
                .until_acceptable()
                .default(Answer::YES)
                .ask()
                .expect("Couldn't ask question.");

            if answer == Answer::NO {
                error!("Commit aborted by user.");
                std::process::exit(1);
            }
            info!("Committing Message...");
        }
    }

    let mut ps_commit = Command::new("git")
        .arg("commit")
        .args(if cli.review { vec!["-e"] } else { vec![] })
        .arg("-F")
        .arg("-")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = ps_commit.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(commit_msg.as_bytes())
            .expect("Failed to write to stdin");
    });

    let commit_output = ps_commit
        .wait_with_output()
        .expect("There was an error when creating the commit.");

    info!("{}", str::from_utf8(&commit_output.stdout).unwrap());

    Ok(())
}
