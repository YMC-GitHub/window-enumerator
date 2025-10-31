#!/bin/bash
# Pure shell CHANGELOG generator with type filtering
# Supports Conventional Commits and Keep a Changelog format

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认配置
REPO_PATH="."
CHANGELOG_FILE="CHANGELOG.md"
INCLUDE_UNRELEASED=true
TARGET_VERSION=""
DRY_RUN=false
AUTO_COMMIT=false
COMMIT_MESSAGE="docs: update changelog"
INCLUDE_TYPES=""
EXCLUDE_TYPES=""

# 提交类型映射
declare -A TYPE_MAP=(
    ["feat"]="Features"
    ["fix"]="Bug Fixes"
    ["docs"]="Documentation"
    ["style"]="Styles"
    ["refactor"]="Code Refactoring"
    ["perf"]="Performance Improvements"
    ["test"]="Tests"
    ["build"]="Build System"
    ["ci"]="Continuous Integration"
    ["chore"]="Chores"
)

# 默认忽略的提交类型
DEFAULT_IGNORE_TYPES="chore test build ci"

# 打印帮助信息
print_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Generate CHANGELOG.md from git history using Conventional Commits.

OPTIONS:
    -u, --unreleased      Include unreleased changes (default: true)
    -v, --version VERSION Generate changelog for specific version only
    -r, --repo PATH       Path to git repository (default: .)
    -n, --dry-run         Dry run, don't write to file
    -c, --commit          Auto-commit changes after generation
    -m, --message TEXT    Custom commit message (default: "docs: update changelog")
    -i, --include TYPES   Only include specified commit types (comma-separated)
    -e, --exclude TYPES   Exclude specified commit types (comma-separated)
    -h, --help            Show this help message

EXAMPLES:
    $0 --unreleased                    # Generate with unreleased changes
    $0 --version v1.0.0                # Generate for specific version
    $0 --dry-run                       # Show what would be generated
    $0 --commit                        # Generate and auto-commit
    $0 --include feat,fix,docs         # Only include features, fixes, and docs
    $0 --exclude chore,test            # Exclude chores and tests
    $0 --include feat --exclude chore  # Only features, excluding chores

VALID TYPES:
    feat, fix, docs, style, refactor, perf, test, build, ci, chore

CONVENTIONAL COMMITS:
    Format: type(scope): description
    Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore
EOF
}

# 日志函数
log_info() {
    echo -e "${BLUE}ℹ ${NC}$1"
}

log_success() {
    echo -e "${GREEN}✓ ${NC}$1"
}

log_warning() {
    echo -e "${YELLOW}⚠ ${NC}$1"
}

log_error() {
    echo -e "${RED}✗ ${NC}$1" >&2
}

# 检查依赖
check_dependencies() {
    local deps=("git" "sed" "awk" "grep")
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" > /dev/null 2>&1; then
            log_error "Required dependency not found: $dep"
            exit 1
        fi
    done
}

# 运行 git 命令
run_git() {
    git -C "$REPO_PATH" "$@"
}

# 验证类型
validate_types() {
    local types="$1"
    local valid_types="feat fix docs style refactor perf test build ci chore"
    
    for type in $(echo "$types" | tr ',' ' '); do
        if ! echo "$valid_types" | grep -qw "$type"; then
            log_error "Invalid commit type: $type"
            log_error "Valid types are: $valid_types"
            exit 1
        fi
    done
}

# 检查是否应该包含提交类型
should_include_type() {
    local commit_type="$1"
    local breaking="$2"
    
    # 如果是破坏性变更，总是包含
    if [ "$breaking" = "true" ]; then
        return 0
    fi
    
    # 如果指定了包含类型，检查是否在包含列表中
    if [ -n "$INCLUDE_TYPES" ]; then
        for include_type in $(echo "$INCLUDE_TYPES" | tr ',' ' '); do
            if [ "$commit_type" = "$include_type" ]; then
                return 0
            fi
        done
        return 1
    fi
    
    # 如果指定了排除类型，检查是否在排除列表中
    if [ -n "$EXCLUDE_TYPES" ]; then
        for exclude_type in $(echo "$EXCLUDE_TYPES" | tr ',' ' '); do
            if [ "$commit_type" = "$exclude_type" ]; then
                return 1
            fi
        done
        return 0
    fi
    
    # 默认行为：使用默认忽略列表
    for ignore_type in $DEFAULT_IGNORE_TYPES; do
        if [ "$commit_type" = "$ignore_type" ]; then
            return 1
        fi
    done
    
    return 0
}

# 获取所有标签
get_tags() {
    run_git tag --sort=-creatordate
}

# 获取标签日期
get_tag_date() {
    local tag="$1"
    run_git log -1 --format=%ai "$tag" | awk '{print $1}'
}

# 获取提交范围
get_commits_between() {
    local start_ref="$1"
    local end_ref="$2"
    
    if [ -z "$start_ref" ]; then
        run_git log --reverse --pretty=format:"%H|%s|%b|%ai|%an" "$end_ref"
    else
        run_git log --reverse --pretty=format:"%H|%s|%b|%ai|%an" "${start_ref}..${end_ref}"
    fi
}

# 解析提交消息
parse_commit_message() {
    local subject="$1"
    local body="$2"
    local hash="$3"
    local author="$4"
    
    # 约定式提交格式: type(scope): description
    if echo "$subject" | grep -qE "^([a-zA-Z]+)(\([^)]+\))?:\ .+$"; then
        local commit_type=$(echo "$subject" | sed -E 's/^([a-zA-Z]+)(\([^)]+\))?:\ .+$/\1/')
        local scope=$(echo "$subject" | sed -E 's/^[a-zA-Z]+(\(([^)]+)\))?:\ .+$/\2/')
        local description=$(echo "$subject" | sed -E 's/^[a-zA-Z]+(\([^)]+\))?:\ //')
        local breaking=false
        
        # 检查破坏性变更
        if echo "$subject" | grep -q "!" || echo "$body" | grep -qi "breaking change"; then
            breaking=true
        fi
        
        echo "$commit_type|$scope|$description|$breaking|${hash:0:8}|$author"
    else
        # 非约定式提交
        echo "other||$subject|false|${hash:0:8}|$author"
    fi
}

# 分类提交
categorize_commits() {
    local commits=()
    while IFS= read -r line; do
        commits+=("$line")
    done
    
    # 初始化类别
    declare -A categorized
    for category in "${TYPE_MAP[@]}"; do
        categorized["$category"]=""
    done
    categorized["Other Changes"]=""
    
    local breaking_changes=()
    
    for commit_line in "${commits[@]}"; do
        IFS='|' read -r hash subject body date author <<< "$commit_line"
        
        # 解析提交消息
        local parsed
        parsed=$(parse_commit_message "$subject" "$body" "$hash" "$author")
        IFS='|' read -r commit_type scope description breaking hash_short author_name <<< "$parsed"
        
        # 检查是否应该包含
        if ! should_include_type "$commit_type" "$breaking"; then
            continue
        fi
        
        # 添加到破坏性变更
        if [ "$breaking" = "true" ]; then
            breaking_changes+=("$description|$hash_short|$author_name")
        fi
        
        # 确定类别
        local category="${TYPE_MAP[$commit_type]:-Other Changes}"
        
        # 构建条目
        local entry="- "
        if [ -n "$scope" ] && [ "$scope" != "null" ]; then
            entry+="**$scope**: "
        fi
        entry+="$description"
        if [ "$breaking" = "true" ]; then
            entry+=" ⚠️ **BREAKING**"
        fi
        if [ -n "$author_name" ]; then
            entry+=" ([@$author_name](https://github.com/$author_name))"
        fi
        entry+=$'\n'
        
        # 添加到类别
        categorized["$category"]="${categorized[$category]}$entry"
    done
    
    # 添加破坏性变更类别
    if [ ${#breaking_changes[@]} -gt 0 ]; then
        categorized["BREAKING CHANGES"]=""
        for bc in "${breaking_changes[@]}"; do
            IFS='|' read -r desc hash_short author_name <<< "$bc"
            categorized["BREAKING CHANGES"]="${categorized[BREAKING CHANGES]}- $desc ([@$author_name](https://github.com/$author_name))"$'\n'
        done
    fi
    
    # 输出分类结果
    for category in "${!categorized[@]}"; do
        local content="${categorized[$category]}"
        if [ -n "$content" ]; then
            echo "===CATEGORY_START==="
            echo "$category"
            echo "===CONTENT_START==="
            printf "%s" "$content"
            echo "===CONTENT_END==="
        fi
    done
}

# 生成版本部分
generate_version_section() {
    local version="$1"
    local date="$2"
    local commits=()
    
    while IFS= read -r line; do
        commits+=("$line")
    done
    
    local section="## $version - $date"$'\n\n'
    
    # 分类提交
    local categorized
    categorized=$(printf '%s\n' "${commits[@]}" | categorize_commits)
    
    # 处理分类结果
    local current_category=""
    local current_content=""
    
    while IFS= read -r line; do
        if [ "$line" = "===CATEGORY_START===" ]; then
            read -r current_category
            continue
        fi
        
        if [ "$line" = "===CONTENT_START===" ]; then
            current_content=""
            while IFS= read -r content_line; do
                if [ "$content_line" = "===CONTENT_END===" ]; then
                    break
                fi
                current_content+="$content_line"$'\n'
            done
            
            section+="### $current_category"$'\n\n'
            section+="$current_content"
            section+=$'\n'
        fi
    done <<< "$categorized"
    
    echo "$section"
}

# 生成未发布部分
generate_unreleased_section() {
    local since_tag="$1"
    local commits=()
    
    while IFS= read -r line; do
        commits+=("$line")
    done < <(get_commits_between "$since_tag" "HEAD")
    
    if [ ${#commits[@]} -eq 0 ]; then
        return
    fi
    
    local section="## [Unreleased]"$'\n\n'
    local categorized
    categorized=$(printf '%s\n' "${commits[@]}" | categorize_commits)
    
    local current_category=""
    local current_content=""
    
    while IFS= read -r line; do
        if [ "$line" = "===CATEGORY_START===" ]; then
            read -r current_category
            continue
        fi
        
        if [ "$line" = "===CONTENT_START===" ]; then
            current_content=""
            while IFS= read -r content_line; do
                if [ "$content_line" = "===CONTENT_END===" ]; then
                    break
                fi
                current_content+="$content_line"$'\n'
            done
            
            section+="### $current_category"$'\n\n'
            section+="$current_content"
            section+=$'\n'
        fi
    done <<< "$categorized"
    
    echo "$section"
}

# 自动提交更改
auto_commit() {
    local changelog_path="$REPO_PATH/$CHANGELOG_FILE"
    
    # 检查是否有更改
    if ! run_git diff --quiet "$CHANGELOG_FILE"; then
        log_info "Auto-committing changelog changes..."
        run_git add "$CHANGELOG_FILE"
        run_git commit -m "$COMMIT_MESSAGE"
        log_success "Committed changelog changes: $COMMIT_MESSAGE"
    else
        log_info "No changes to commit"
    fi
}

# 生成完整的 CHANGELOG
generate_changelog() {
    local changelog="# Changelog"$'\n\n'
    changelog+="All notable changes to this project will be documented in this file."$'\n\n'
    changelog+="The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),"$'\n'
    changelog+="and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)."$'\n\n'
    
    # 显示过滤信息
    if [ -n "$INCLUDE_TYPES" ]; then
        changelog+="> **Filter**: Only including types: $INCLUDE_TYPES"$'\n\n'
    elif [ -n "$EXCLUDE_TYPES" ]; then
        changelog+="> **Filter**: Excluding types: $EXCLUDE_TYPES"$'\n\n'
    fi
    
    # 获取标签
    local tags=()
    while IFS= read -r tag; do
        tags+=("$tag")
    done < <(get_tags)
    
    # 未发布部分
    if [ "$INCLUDE_UNRELEASED" = "true" ]; then
        local last_tag=""
        if [ ${#tags[@]} -gt 0 ]; then
            last_tag="${tags[0]}"
        fi
        
        local unreleased_section
        unreleased_section=$(generate_unreleased_section "$last_tag")
        
        if [ -n "$unreleased_section" ]; then
            changelog+="$unreleased_section"
        fi
    fi
    
    # 各版本历史
    for ((i=0; i<${#tags[@]}; i++)); do
        local tag="${tags[i]}"
        local date
        date=$(get_tag_date "$tag")
        
        # 如果指定了目标版本，只处理该版本
        if [ -n "$TARGET_VERSION" ] && [ "$tag" != "$TARGET_VERSION" ]; then
            continue
        fi
        
        # 获取前一个标签
        local prev_tag=""
        if [ $i -lt $((${#tags[@]} - 1)) ]; then
            prev_tag="${tags[i+1]}"
        fi
        
        # 获取提交
        local commits=()
        while IFS= read -r line; do
            commits+=("$line")
        done < <(get_commits_between "$prev_tag" "$tag")
        
        local version_section
        version_section=$(generate_version_section "$tag" "$date" "$(printf '%s\n' "${commits[@]}")")
        changelog+="$version_section"
        
        # 如果指定了目标版本，处理完就退出
        if [ -n "$TARGET_VERSION" ]; then
            break
        fi
    done
    
    echo "$changelog"
}

# 主函数
main() {
    # 解析命令行参数
    while [ $# -gt 0 ]; do
        case $1 in
            -u|--unreleased)
                INCLUDE_UNRELEASED=true
                shift
                ;;
            -v|--version)
                TARGET_VERSION="$2"
                shift 2
                ;;
            -r|--repo)
                REPO_PATH="$2"
                shift 2
                ;;
            -n|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -c|--commit)
                AUTO_COMMIT=true
                shift
                ;;
            -m|--message)
                COMMIT_MESSAGE="$2"
                shift 2
                ;;
            -i|--include)
                INCLUDE_TYPES="$2"
                validate_types "$INCLUDE_TYPES"
                shift 2
                ;;
            -e|--exclude)
                EXCLUDE_TYPES="$2"
                validate_types "$EXCLUDE_TYPES"
                shift 2
                ;;
            -h|--help)
                print_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_help
                exit 1
                ;;
        esac
    done
    
    # 检查依赖
    check_dependencies
    
    # 检查仓库
    if ! run_git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "Not a git repository: $REPO_PATH"
        exit 1
    fi
    
    log_info "Generating changelog for: $(basename "$(run_git rev-parse --show-toplevel)")"
    
    # 显示过滤信息
    if [ -n "$INCLUDE_TYPES" ]; then
        log_info "Filter: Only including types: $INCLUDE_TYPES"
    elif [ -n "$EXCLUDE_TYPES" ]; then
        log_info "Filter: Excluding types: $EXCLUDE_TYPES"
    fi
    
    # 生成 CHANGELOG
    local changelog_content
    changelog_content=$(generate_changelog)
    
    if [ "$DRY_RUN" = "true" ]; then
        log_success "Dry run - generated changelog:"
        echo "=========================================="
        echo "$changelog_content"
        echo "=========================================="
    else
        # 写入文件
        local changelog_path="$REPO_PATH/$CHANGELOG_FILE"
        echo "$changelog_content" > "$changelog_path"
        log_success "CHANGELOG.md has been updated at: $changelog_path"
        
        # 自动提交
        if [ "$AUTO_COMMIT" = "true" ]; then
            auto_commit
        fi
        
        # 显示统计信息
        local tags=()
        while IFS= read -r tag; do
            tags+=("$tag")
        done < <(get_tags)
        
        if [ ${#tags[@]} -gt 0 ]; then
            local last_tag="${tags[0]}"
            local commits_since_last=()
            while IFS= read -r line; do
                commits_since_last+=("$line")
            done < <(get_commits_between "$last_tag" "HEAD")
            log_info "Since $last_tag: ${#commits_since_last[@]} commits"
        fi
    fi
}

# 运行主函数
main "$@"