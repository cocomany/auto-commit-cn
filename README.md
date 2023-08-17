# 自动生成的提交信息
本项目Folk自 [m1guelpf/auto-commit](https://github.com/m1guelpf/auto-commit)， 为适配墙内使用，修改了部分代码，并添加了中文支持。

这是一个使用[Rust](https://www.rust-lang.org/)构建的CLI工具，利用[OpenAI的GPT-3.5](https://platform.openai.com/overview)从您的暂存更改中生成提交信息。

## 安装

您可以从[最新版本发布页面](https://github.com/cocomany/auto-commit-cn/releases/latest)下载适用于您操作系统的二进制文件。
将此文件放到系统环境变量路径下。比如Windows下的 C:\Windows,  Linux下的 /usr/bin/ 

## 使用

`auto-commit`使用GPT-3.5进行工作。要使用它，请从[您的控制台](https://platform.openai.com/)获取API密钥，并将其保存到`OPENAI_API_KEY`中（您还可以将其保存在bash/zsh配置文件中以在会话之间保持持久性）。

```bash
export OPENAI_API_KEY='sk-XXXXXXXX'
```

配置好环境后，通过运行例如 `git add .` 来暂存一些更改，然后运行 `auto-commit`。

当然，`auto-commit`还包括一些选项，用于在提交之前编辑消息或仅将消息打印到终端。

```sh
$ auto-commit --help
自动生成提交信息。

用法: auto-commit [OPTIONS]

选项:
  -v, --verbose...  每次发生事件时输出更多信息
  -q, --quiet...    每次发生事件时输出较少信息
      --dry-run     输出生成的消息，但不创建提交。
  -r, --review      在提交之前编辑生成的提交消息。
  -h, --help        打印帮助信息
  -V, --version     打印版本信息
```

## 开发

确保您已安装最新版本的Rust（使用[rustup](https://rustup.rs/)）。然后，您可以通过运行`cargo build`构建项目，并使用`cargo run`运行它。

## 许可证

该项目在MIT许可下开源。有关更多信息，请参阅[许可证文件](LICENSE)。 
