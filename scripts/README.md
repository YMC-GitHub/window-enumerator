## 编码助手
```sh
sh ./scripts/pre.ai.sh -i src -o src_code.txt

sh ./scripts/pre.ai.sh -i src -o src_code.txt -a Cargo.toml

sh ./scripts/pre.ai.sh -i src -o src_code.txt -a Cargo.toml --ignore gui.rs




# workspace
sh ./scripts/pre.ai.sh -i crates/cli -o src_code.txt -a Cargo.toml --ignore gui.rs

sh ./scripts/pre.ai.sh -i crates/dll -o src_code_b.txt -a Cargo.toml

sh ./scripts/pre.ai.sh -i crates/shared -o src_code_c.txt -a Cargo.toml

# to analyze the cli code
sh ./scripts/pre.ai.sh -i src/cli.rs -o src_cli_code.txt

# to analyze the main code
sh ./scripts/pre.ai.sh -i src/main.rs -o src_main_code.txt

# to analyze the output code
sh ./scripts/pre.ai.sh -i src/output.rs -o src_output_code.txt

# to analyze the output code
sh ./scripts/pre.ai.sh -i src/utils.rs -o src_utils_code.txt

# 


# to analyze the platform code
sh ./scripts/pre.ai.sh -i src/platform -o src_platform_code.txt -a src/main.rs

sh ./scripts/pre.ai.sh -o src_changed_code.txt -i src/cli_utils.rs  -a src/main.rs -a src/features/always_on_top.rs

sh ./scripts/pre.ai.sh -o src_changed_code.txt -i src/cli_utils.rs  -a src/main.rs -a src/features/transparency.rs

# 窗口截图修复
sh ./scripts/pre.ai.sh -i src/platform/windows/screenshot.rs -o src_main_code.txt -a Cargo.toml 

sh ./scripts/pre.ai.sh -i src/platform/windows/screenshot.rs -o src_main_code.txt -a Cargo.toml -a src/platform/windows/enumeration.rs



# sh -c "rm src_*.txt"
```

## 为 linux 平台构建

### 容器环境-musl版
```bash
./scripts/local-build-scratch-in-docker.sh build_only --china-mirror --rust-mirror ustc
./scripts/local-build-scratch-in-docker.sh extract --china-mirror --rust-mirror ustc

# move to dist diretory and add musl suffix
# mv dist/pass-craft dist/pass-craft-musl

# move to dist diretory and add x86_64-unknown-linux-musl subdiretory
# mkdir -p dist/x86_64-unknown-linux-musl
# mv dist/pass-craft dist/x86_64-unknown-linux-musl/pass-craft

```

## 为 window 平台构建

### 真机环境-msvc版
```powershell
cargo build --release --target x86_64-pc-windows-msvc
# dir target\x86_64-pc-windows-msvc\release\pass-craft.exe
# target\x86_64-pc-windows-msvc\release\pass-craft.exe --version

# get file size
$fileSize = (Get-Item -Path "target\x86_64-pc-windows-msvc\release\pass-craft.exe").Length
$fileSizeInMB = [Math]::Round($fileSize / 1MB, 2)
Write-Output "File size: $fileSizeInMB MB"

# run this file

# copy this file to dist/pass-craft.exe
# make dist directory
# New-Item -ItemType Directory -Force -Path "dist" | Out-Null
# Copy-Item -Path "target\x86_64-pc-windows-msvc\release\pass-craft.exe" -Destination "dist\pass-craft.exe"

# move to dist diretory and add msvc suffix
# mv dist/pass-craft.exe dist/pass-craft-msvc.exe

# move to dist diretory and add x86_64-pc-windows-msvc subdiretory
New-Item -ItemType Directory -Force -Path "dist\x86_64-pc-windows-msvc" | Out-Null
Copy-Item -Path "target\x86_64-pc-windows-msvc\release\pass-craft.exe" -Destination "dist\x86_64-pc-windows-msvc\pass-craft.exe"  | Out-Null

```

### 容器环境-gnu版
- 使用轻量化镜像构建gnu版本
- 使用国内镜像为系统下载工具提速
- 使用国内镜像加速cargo下载依赖
- 使用多阶段构建
- 参考文件 dockerfilexxx
- 使用 rust:1.90-alpine3.20
- rust:1.90	 vs rust:slim vs rust:alpine vs rust:1.90-alpine3.20

```bash
./scripts/local-build-scratch-in-docker.sh build_only --china-mirror --rust-mirror ustc --tag window-gnu --target output --dockerfile Dockerfile.window.gnu.alpine
./scripts/local-build-scratch-in-docker.sh extract_window_gnu_binary --china-mirror --rust-mirror ustc --tag window-gnu --target output --dockerfile Dockerfile.window.gnu.alpine

# move to dist diretory and add gnu suffix
# mv dist/pass-craft.exe dist/pass-craft-gnu.exe

# move to dist diretory and add x86_64-pc-windows-gnu subdiretory
# mkdir -p dist/x86_64-pc-windows-gnu
# mv dist/pass-craft.exe dist/x86_64-pc-windows-gnu/pass-craft.exe

```

## 打包为镜像,并发布到 docker hub
- docker.io
- ghcr.io
- ~~mcr.microsoft.com~~
```bash
yours touch .github/workflows/push-docker-io.yml

# sh -c "rm -r /.github"

source .env
./scripts/push-docker-io.sh

docker publish pass-craft:latest
docker publish pass-craft:optimized
docker publish pass-craft:alpine
docker publish pass-craft:scratch

```

## 本地容器中构建镜像 - 研发
```bash
# docker logs pass-craft
docker stop $(docker ps -a | grep pass-craft | awk '{print $1}')
docker rm $(docker ps -a | grep pass-craft | awk '{print $1}')
docker rmi $(docker images | grep pass-craft | grep -v dev | awk '{print $3}')
docker rmi $(docker images -f "dangling=true" -q)
# docker images --filter "reference=pass-craft" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}" | (read -r header; echo "$header"; sort -hk3 ) | head -n 10


# 构建生产镜像
docker build --target runtime -t pass-craft:scratch .

# 使用国内镜像构建生产版本
docker build --target runtime --build-arg USE_CHINA_MIRROR=true -t pass-craft:scratch .

# docker run -d --name pass-craft --restart unless-stopped --env-file .env pass-craft:scratch


# ./scripts/local-build-scratch-in-docker.sh --china-mirror --rust-mirror ustc --tag scratch --target runtime

# 完整构建（默认）
./scripts/local-build-scratch-in-docker.sh

# 快速构建
./scripts/local-build-scratch-in-docker.sh build_quick

# 仅构建镜像
./scripts/local-build-scratch-in-docker.sh build_only

# 仅分析镜像
./scripts/local-build-scratch-in-docker.sh analyze

# 仅测试功能
./scripts/local-build-scratch-in-docker.sh test

# 仅提取二进制文件
./scripts/local-build-scratch-in-docker.sh extract

# 清理资源
./scripts/local-build-scratch-in-docker.sh cleanup

# 显示帮助
./scripts/local-build-scratch-in-docker.sh help

./scripts/local-build-scratch-in-docker.sh show_config --china-mirror --rust-mirror ustc --target output


./scripts/local-build-scratch-in-docker.sh build_only --china-mirror --rust-mirror ustc


./scripts/local-build-scratch-in-docker.sh build_only --china-mirror --rust-mirror ustc
./scripts/local-build-scratch-in-docker.sh extract --china-mirror --rust-mirror ustc


./scripts/local-build-scratch-in-docker.sh show_config --china-mirror --rust-mirror ustc --tag window-gnu --target output --dockerfile Dockerfile.window.gnu.alpine

# 
./scripts/local-build-scratch-in-docker.sh build_only --china-mirror --rust-mirror ustc --tag window-gnu --target output --dockerfile Dockerfile.window.gnu.alpine
./scripts/local-build-scratch-in-docker.sh extract_window_gnu_binary --china-mirror --rust-mirror ustc --tag window-gnu --target output --dockerfile Dockerfile.window.gnu.alpine

# docker run --rm -v $(pwd)/dist:/app pass-craft:window-gnu sh
# docker run --rm -it -v $(pwd)/dist:/app pass-craft:window-gnu sh
# docker run --rm -it -v $(pwd)/dist:/app --entrypoint="" pass-craft:window-gnu sh
# docker run --rm --entrypoint="" pass-craft:window-gnu ls 
# docker run --rm --entrypoint="" pass-craft:window-gnu tail -f /dev/null
```

## 变更日志
```
# 生成完整的变更日志（包括未发布更改）
./scripts/generate-changelog.sh --unreleased

# 只生成已发布版本的变更日志
./scripts/generate-changelog.sh

# 生成特定版本的变更日志
./scripts/generate-changelog.sh --version v0.4.0

# 干运行（只显示不写入文件）
./scripts/generate-changelog.sh --dry-run

# 指定其他仓库路径
./scripts/generate-changelog.sh --repo /path/to/other/repo --unreleased


# 只包含功能、修复和文档
./scripts/generate-changelog.sh --include feat,fix,docs

# 只包含功能
./scripts/generate-changelog.sh --include feat

# 排除杂项和测试
./scripts/generate-changelog.sh --exclude chore,test

# 排除构建和CI
./scripts/generate-changelog.sh --exclude build,ci

# 排除样式修改
./scripts/generate-changelog.sh --exclude style

# 只包含功能和重构
./scripts/generate-changelog.sh --include feat,refactor

# 只生成重要变更（功能、修复、破坏性变更）
./scripts/generate-changelog.sh --include feat,fix

# 生成发布版本，排除杂项
./scripts/generate-changelog.sh --version v1.0.0 --exclude chore,test,build,ci

# 生成开发版变更日志，只关注功能
./scripts/generate-changelog.sh --unreleased --include feat

# 完整过滤并自动提交
./scripts/generate-changelog.sh --include feat,fix,docs --commit --message "chore: update production changelog"
```