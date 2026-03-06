#!/usr/bin/env bash

zero_sha() {
  [ "$1" = "0000000000000000000000000000000000000000" ]
}

is_dry_run() {
  [ "${HOOK_DRY_RUN:-0}" = "1" ]
}

is_doc_path() {
  case "$1" in
    *.md|*.txt|*.rst|LICENSE|docs/*) return 0 ;;
    *) return 1 ;;
  esac
}

is_frontend_path() {
  case "$1" in
    src/*|public/*|index.html|vite.config.ts|tsconfig.json|tsconfig.node.json|eslint.config.js|package.json|package-lock.json|bun.lock)
      return 0
      ;;
    *) return 1 ;;
  esac
}

is_rust_path() {
  case "$1" in
    src-tauri/*) return 0 ;;
    *) return 1 ;;
  esac
}

is_frontend_test_path() {
  case "$1" in
    src/*.test.ts|src/*.test.tsx|src/*.test.js|src/*.test.jsx|src/*.spec.ts|src/*.spec.tsx|src/*.spec.js|src/*.spec.jsx|src/__tests__/*)
      return 0
      ;;
    *) return 1 ;;
  esac
}

is_frontend_config_path() {
  case "$1" in
    package.json|package-lock.json|bun.lock|vite.config.ts|tsconfig.json|tsconfig.node.json|eslint.config.js|src/test/*)
      return 0
      ;;
    *) return 1 ;;
  esac
}

is_rust_integration_test_path() {
  case "$1" in
    src-tauri/tests/*.rs)
      return 0
      ;;
    *) return 1 ;;
  esac
}

is_rust_shared_test_path() {
  case "$1" in
    src-tauri/tests/common/*)
      return 0
      ;;
    *) return 1 ;;
  esac
}

is_hook_path() {
  case "$1" in
    .gitattributes|.husky/*|scripts/git-hooks/*) return 0 ;;
    *) return 1 ;;
  esac
}

needs_support_matrix_check_path() {
  case "$1" in
    docs/SUPPORT_MATRIX.md|src-tauri/src/models/registry.rs|src-tauri/src/bin/gen_docs.rs)
      return 0
      ;;
    *) return 1 ;;
  esac
}

collect_staged_files() {
  if [ -n "${HOOK_STAGED_FILES_OVERRIDE:-}" ]; then
    printf "%s\n" "$HOOK_STAGED_FILES_OVERRIDE" | sed '/^$/d' | sort -u
    return
  fi

  git diff --cached --name-only --diff-filter=ACMR | sed '/^$/d' | sort -u
}

fallback_push_base() {
  git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo "origin/main"
}

collect_push_files() {
  if [ -n "${HOOK_PUSH_FILES_OVERRIDE:-}" ]; then
    printf "%s\n" "$HOOK_PUSH_FILES_OVERRIDE" | sed '/^$/d' | sort -u
    return
  fi

  local stdin_file
  stdin_file=$(mktemp)
  cat > "$stdin_file"

  if [ ! -s "$stdin_file" ]; then
    git diff --name-only "$(fallback_push_base)"...HEAD 2>/dev/null || git show --pretty="" --name-only HEAD
    rm -f "$stdin_file"
    return
  fi

  while IFS=' ' read -r local_ref local_sha remote_ref remote_sha; do
    [ -z "$local_ref" ] && continue

    if zero_sha "$local_sha"; then
      continue
    fi

    if ! zero_sha "$remote_sha"; then
      git diff --name-only "$remote_sha..$local_sha"
      continue
    fi

    local base_ref merge_base
    base_ref=$(fallback_push_base)
    merge_base=""

    if git rev-parse --verify "$base_ref" >/dev/null 2>&1; then
      merge_base=$(git merge-base "$local_sha" "$base_ref" 2>/dev/null || true)
    fi

    if [ -n "$merge_base" ]; then
      git diff --name-only "$merge_base..$local_sha"
    else
      git show --pretty="" --name-only "$local_sha"
    fi
  done < "$stdin_file" | sed '/^$/d' | sort -u

  rm -f "$stdin_file"
}

classify_paths() {
  HAS_ANY=0
  HAS_DOCS_ONLY=1
  HAS_FRONTEND=0
  HAS_RUST=0
  HAS_HOOKS=0
  HAS_SUPPORT_MATRIX=0
  HAS_LINT_STAGED=0

  while IFS= read -r file; do
    [ -z "$file" ] && continue

    HAS_ANY=1

    if ! is_doc_path "$file"; then
      HAS_DOCS_ONLY=0
    fi

    if is_frontend_path "$file"; then
      HAS_FRONTEND=1
    fi

    if is_rust_path "$file"; then
      HAS_RUST=1
    fi

    if is_hook_path "$file"; then
      HAS_HOOKS=1
    fi

    if needs_support_matrix_check_path "$file"; then
      HAS_SUPPORT_MATRIX=1
    fi

    case "$file" in
      *.ts|*.tsx|*.css|*.json|*.md) HAS_LINT_STAGED=1 ;;
    esac
  done

  if [ "$HAS_ANY" -eq 0 ]; then
    HAS_DOCS_ONLY=0
  fi
}

describe_detected_scopes() {
  local scopes=()

  if [ "$HAS_DOCS_ONLY" -eq 1 ]; then
    scopes+=("docs-only")
  else
    [ "$HAS_FRONTEND" -eq 1 ] && scopes+=("frontend")
    [ "$HAS_RUST" -eq 1 ] && scopes+=("rust")
    [ "$HAS_HOOKS" -eq 1 ] && scopes+=("hooks")
    [ "$HAS_SUPPORT_MATRIX" -eq 1 ] && scopes+=("support-matrix")
  fi

  if [ "${#scopes[@]}" -eq 0 ]; then
    scopes+=("misc")
  fi

  printf "%s" "${scopes[*]}"
}

hook_shell_files_from_paths() {
  local include_all_hooks=0
  local files=()

  while IFS= read -r file; do
    case "$file" in
      .gitattributes)
        include_all_hooks=1
        ;;
      .husky/*|scripts/git-hooks/*)
        [ -f "$file" ] && files+=("$file")
        ;;
    esac
  done

  if [ "$include_all_hooks" -eq 1 ]; then
    files+=(.husky/pre-commit .husky/pre-push scripts/git-hooks/common.sh)
  fi

  printf "%s\n" "${files[@]}" | sed '/^$/d' | sort -u
}

rust_files_from_paths() {
  while IFS= read -r file; do
    case "$file" in
      *.rs)
        [ -f "$file" ] && printf "%s\n" "$file"
        ;;
    esac
  done | sort -u
}

frontend_eslint_files_from_paths() {
  while IFS= read -r file; do
    case "$file" in
      *.ts|*.tsx)
        if is_frontend_path "$file" && [ -f "$file" ]; then
          printf "%s\n" "$file"
        fi
        ;;
    esac
  done | sort -u
}

frontend_test_files_from_paths() {
  while IFS= read -r file; do
    if is_frontend_test_path "$file" && [ -f "$file" ]; then
      printf "%s\n" "$file"
    fi
  done | sort -u
}

frontend_source_files_from_paths() {
  while IFS= read -r file; do
    case "$file" in
      *.ts|*.tsx|*.js|*.jsx)
        if is_frontend_path "$file" && ! is_frontend_test_path "$file" && ! is_frontend_config_path "$file" && [ -f "$file" ]; then
          printf "%s\n" "$file"
        fi
        ;;
    esac
  done | sort -u
}

frontend_named_test_files_from_source_paths() {
  while IFS= read -r file; do
    local relative_dir base_name extension
    relative_dir="${file#src/}"
    relative_dir="$(dirname "$relative_dir")"
    [ "$relative_dir" = "." ] && relative_dir=""

    base_name="$(basename "$file")"
    extension="${base_name##*.}"
    base_name="${base_name%.*}"

    for candidate in \
      "src/__tests__/${relative_dir:+$relative_dir/}${base_name}.test.${extension}" \
      "src/__tests__/${relative_dir:+$relative_dir/}${base_name}.spec.${extension}" \
      "src/${relative_dir:+$relative_dir/}${base_name}.test.${extension}" \
      "src/${relative_dir:+$relative_dir/}${base_name}.spec.${extension}"
    do
      if [ -f "$candidate" ]; then
        printf "%s\n" "$candidate"
      fi
    done
  done | sort -u
}

has_frontend_source_without_named_tests() {
  while IFS= read -r file; do
    [ -z "$file" ] && continue

    local relative_dir base_name extension found_match=0
    relative_dir="${file#src/}"
    relative_dir="$(dirname "$relative_dir")"
    [ "$relative_dir" = "." ] && relative_dir=""

    base_name="$(basename "$file")"
    extension="${base_name##*.}"
    base_name="${base_name%.*}"

    for candidate in \
      "src/__tests__/${relative_dir:+$relative_dir/}${base_name}.test.${extension}" \
      "src/__tests__/${relative_dir:+$relative_dir/}${base_name}.spec.${extension}" \
      "src/${relative_dir:+$relative_dir/}${base_name}.test.${extension}" \
      "src/${relative_dir:+$relative_dir/}${base_name}.spec.${extension}"
    do
      if [ -f "$candidate" ]; then
        found_match=1
        break
      fi
    done

    if [ "$found_match" -eq 0 ]; then
      return 0
    fi
  done

  return 1
}

has_frontend_config_changes() {
  while IFS= read -r file; do
    if is_frontend_config_path "$file"; then
      return 0
    fi
  done

  return 1
}

rust_integration_test_targets_from_paths() {
  while IFS= read -r file; do
    if is_rust_integration_test_path "$file" && ! is_rust_shared_test_path "$file"; then
      basename "$file" .rs
    fi
  done | sort -u
}

has_rust_non_integration_changes() {
  while IFS= read -r file; do
    if is_rust_path "$file" && ! is_rust_integration_test_path "$file"; then
      return 0
    fi

    if is_rust_shared_test_path "$file"; then
      return 0
    fi
  done

  return 1
}