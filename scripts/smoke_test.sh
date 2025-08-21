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
if ! echo "$langs_json" | jq -e '.[] | select(.language=="kotlin")' >/dev/null; then
  echo "Kotlin not available according to /languages" >&2
  echo "$langs_json" | jq '.'
  exit 1
fi

echo "Kotlin detected in /languages"

CODE=$(cat <<'KT'
fun main() {
    val a = readLine()!!.toInt()
    val b = readLine()!!.toInt()
    println(a + b)
}
KT
)

# Test input with newline
input=$'10\n20\n'
expected_output=$'30\n'

payload=$(jq -n --arg lang kotlin --arg code "$CODE" --arg input "$input" --arg expected "$expected_output" '{
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
  st=$(echo "$status_json" | jq -r '.status')
  case "$st" in
    completed)
      echo "Job completed"
      echo "$status_json" | jq '.'
      passed=$(echo "$status_json" | jq -r '.result.results[0].passed')
      ok=$(echo "$status_json" | jq -r '.result.results[0].ok')
      compiled=$(echo "$status_json" | jq -r '.result.compiled')
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
      echo "$status_json" | jq '.'
      exit 1
      ;;
    queued|running)
      sleep 0.5
      ;;
    *)
      echo "Unexpected status: $st" >&2
      echo "$status_json" | jq '.'
      exit 1
      ;;
  esac
done

echo "Timeout waiting for job completion" >&2
exit 1
