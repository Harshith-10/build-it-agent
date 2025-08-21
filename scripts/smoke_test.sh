#!/usr/bin/env bash
set -euo pipefail

HOST="http://127.0.0.1:8910"

# Check server health
if ! curl -fsS "$HOST/health" >/dev/null ; then
  echo "Executor server not reachable at $HOST" >&2
  exit 1
fi

# Check languages
langs_json=$(curl -fsS "$HOST/languages")
if ! echo "$langs_json" | jq -e '.[] | select(.language=="rust")' >/dev/null; then
  echo "Rust not available according to /languages" >&2
  echo "$langs_json" | jq '.'
  exit 1
fi

echo "Rust detected in /languages"

CODE=$(cat <<'RS'
fn main() {
  let mut a = String::new();
  let mut b = String::new();
  std::io::stdin().read_line(&mut a).unwrap();
  std::io::stdin().read_line(&mut b).unwrap();
  let a: i32 = a.trim().parse().unwrap();
  let b: i32 = b.trim().parse().unwrap();
  println!("{}", a + b);
}
RS
)

# sanitize: read JSON from stdin and escape literal control characters that appear
# unescaped inside JSON string values (newlines, tabs, CRs, other C0 controls).
# This uses a small Python state machine to avoid choking jq on malformed JSON
# where the server returned raw newlines inside strings.
sanitize() {
  tmp=$(mktemp -t smoke_sanitize.XXXXXX.py)
  cat > "$tmp" <<'PY'
import sys
s = sys.stdin.read()
out = []
in_str = False
esc = False
for ch in s:
  if in_str:
    if esc:
      out.append(ch)
      esc = False
    else:
      if ch == '\\':
        out.append(ch)
        esc = True
      elif ch == '"':
        out.append(ch)
        in_str = False
      elif ch == '\n':
        out.append('\\n')
      elif ch == '\r':
        out.append('\\r')
      elif ch == '\t':
        out.append('\\t')
      else:
        if ord(ch) < 32:
          out.append('\\u%04x' % ord(ch))
        else:
          out.append(ch)
  else:
    out.append(ch)
    if ch == '"':
      in_str = True
sys.stdout.write(''.join(out))
PY
  python3 "$tmp"
  rc=$?
  rm -f "$tmp"
  return $rc
}

# Test input with newline
input=$'10\n20\n'
expected_output=$'30\n'

payload=$(jq -n --arg lang rust --arg code "$CODE" --arg input "$input" --arg expected "$expected_output" '{
  language: $lang,
  code: $code,
  testcases: [ { id: 1, input: $input, expected: $expected } ]
}')

echo "Submitting job..."
submit=$(curl -fsS -H 'Content-Type: application/json' -d "$payload" "$HOST/execute")
job_id=$(echo "$submit" | jq -r '.id')
if [[ -z "$job_id" || "$job_id" == "null" ]]; then
  echo "Failed to get job id from response:" >&2
  echo "$submit" | jq '.' || echo "$submit"
  exit 1
fi

echo "Job ID: $job_id"

# Poll for completion (up to ~15s)
for i in {1..30}; do
  status_json=$(curl -fsS "$HOST/status/$job_id")
  st=$(echo "$status_json" | sanitize | jq -r '.status')
  case "$st" in
    completed)
      echo "Job completed"
  echo "$status_json" | sanitize | jq '.'
  passed=$(echo "$status_json" | sanitize | jq -r '.result.results[0].passed')
  ok=$(echo "$status_json" | sanitize | jq -r '.result.results[0].ok')
  compiled=$(echo "$status_json" | sanitize | jq -r '.result.compiled')
      if [[ "$passed" == "true" && "$ok" == "true" ]]; then
        echo "Smoke test PASS (compiled=$compiled)"
        exit 0
      else
        echo "Smoke test FAIL: passed=$passed ok=$ok" >&2
        exit 1
      fi
      ;;
    error)
      echo "Job error:" >&2
  echo "$status_json" | sanitize | jq '.'
      exit 1
      ;;
    queued|running)
      sleep 0.5
      ;;
    *)
      echo "Unexpected status: $st" >&2
  echo "$status_json" | sanitize | jq '.'
      exit 1
      ;;
  esac
done

echo "Timeout waiting for job completion" >&2
exit 1
